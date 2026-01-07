use std::cell::RefCell;
use std::ptr;
use std::time::{Duration, Instant};

use objc2::rc::Retained;
use objc2_app_kit::NSPopUpMenuWindowLevel;
use objc2_core_foundation::{CFType, CGPoint, CGRect, CGSize};
use objc2_core_graphics::CGContext;
use objc2_quartz_core::{CALayer, CATransaction};
use tracing::warn;

use crate::sys::cgs_window::{CgsWindow, CgsWindowError};
use crate::sys::skylight::{
    CFRelease, G_CONNECTION, SLSFlushWindowContentRegion, SLWindowContextCreate,
};

unsafe extern "C" {
    fn CGContextFlush(ctx: *mut CGContext);
    fn CGContextClearRect(ctx: *mut CGContext, rect: CGRect);
    fn CGContextSaveGState(ctx: *mut CGContext);
    fn CGContextRestoreGState(ctx: *mut CGContext);
    fn CGContextTranslateCTM(ctx: *mut CGContext, tx: f64, ty: f64);
    fn CGContextScaleCTM(ctx: *mut CGContext, sx: f64, sy: f64);
}

#[derive(Debug, Clone, Copy)]
pub struct BorderColor {
    pub r: f64,
    pub g: f64,
    pub b: f64,
    pub a: f64,
}

impl BorderColor {
    pub fn to_nscolor(&self) -> Retained<objc2_app_kit::NSColor> {
        objc2_app_kit::NSColor::colorWithRed_green_blue_alpha(self.r, self.g, self.b, self.a)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct BorderConfig {
    pub width: f64,
    pub color: BorderColor,
    pub roundness: f64,
}

impl Default for BorderConfig {
    fn default() -> Self {
        Self {
            width: 2.0,
            color: BorderColor {
                r: 51.0 / 255.0,
                g: 204.0 / 255.0,
                b: 1.0,
                a: 1.0,
            },
            roundness: 8.0,
        }
    }
}

impl From<&crate::common::config::WindowBorderSettings> for BorderConfig {
    fn from(config: &crate::common::config::WindowBorderSettings) -> Self {
        Self {
            width: config.width,
            color: BorderColor {
                r: config.color.r,
                g: config.color.g,
                b: config.color.b,
                a: config.color.a,
            },
            roundness: config.roundness,
        }
    }
}

fn ease_in_out(t: f64) -> f64 {
    if t < 0.5 {
        (1.0 - f64::sqrt(1.0 - f64::powi(2.0 * t, 2))) / 2.0
    } else {
        (f64::sqrt(1.0 - f64::powi(-2.0 * t + 2.0, 2)) + 1.0) / 2.0
    }
}

fn blend(a: f64, b: f64, s: f64) -> f64 {
    (1.0 - s) * a + s * b
}

fn interpolate_frame(from: CGRect, to: CGRect, t: f64) -> CGRect {
    let s = ease_in_out(t);
    CGRect {
        origin: CGPoint {
            x: blend(from.origin.x, to.origin.x, s),
            y: blend(from.origin.y, to.origin.y, s),
        },
        size: CGSize {
            width: blend(from.size.width, to.size.width, s),
            height: blend(from.size.height, to.size.height, s),
        },
    }
}

pub struct FocusBorderWindow {
    frame: RefCell<CGRect>,
    target_frame: RefCell<CGRect>,
    start_frame: RefCell<CGRect>,
    animation_start: RefCell<Option<Instant>>,
    animation_duration: RefCell<Duration>,
    root_layer: Retained<CALayer>,
    cgs_window: CgsWindow,
    config: RefCell<BorderConfig>,
    visible: RefCell<bool>,
    animating: RefCell<bool>,
}

impl FocusBorderWindow {
    pub fn new() -> Result<Self, CgsWindowError> {
        let frame = CGRect::new(CGPoint::new(0.0, 0.0), CGSize::new(100.0, 100.0));

        let root_layer = CALayer::layer();
        root_layer.setFrame(CGRect::new(
            CGPoint::new(0.0, 0.0),
            CGSize::new(frame.size.width, frame.size.height),
        ));

        let cgs_window = CgsWindow::new(frame)?;
        if let Err(err) = cgs_window.set_opacity(false) {
            warn!(error=?err, "failed to set border window opacity");
        }
        if let Err(err) = cgs_window.set_alpha(1.0) {
            warn!(error=?err, "failed to set border window alpha");
        }
        if let Err(err) = cgs_window.set_level(NSPopUpMenuWindowLevel as i32) {
            warn!(error=?err, "failed to set border window level");
        }

        Ok(Self {
            frame: RefCell::new(frame),
            target_frame: RefCell::new(frame),
            start_frame: RefCell::new(frame),
            animation_start: RefCell::new(None),
            animation_duration: RefCell::new(Duration::from_secs(0)),
            root_layer,
            cgs_window,
            config: RefCell::new(BorderConfig::default()),
            visible: RefCell::new(false),
            animating: RefCell::new(false),
        })
    }

    pub fn set_config(&self, config: BorderConfig) {
        *self.config.borrow_mut() = config;
        self.update_layer();
        self.present();
    }

    pub fn show(&self, rect: CGRect) {
        *self.target_frame.borrow_mut() = rect;
        *self.visible.borrow_mut() = true;

        if let Err(err) = self.cgs_window.set_shape(rect) {
            warn!(error=?err, "failed to set border window shape");
        }
        self.root_layer.setFrame(CGRect::new(
            CGPoint::new(0.0, 0.0),
            CGSize::new(rect.size.width, rect.size.height),
        ));

        *self.frame.borrow_mut() = rect;

        if let Err(err) = self.cgs_window.set_level(NSPopUpMenuWindowLevel as i32) {
            warn!(error=?err, "failed to set border window level");
        }

        self.update_layer();
        self.present();

        if let Err(err) = self.cgs_window.order_above(None) {
            warn!(error=?err, "failed to order border window above");
        }
    }

    pub fn show_with_animation(&self, rect: CGRect, duration: Duration) {
        let current_frame = *self.frame.borrow();
        let target_frame = rect;

        *self.target_frame.borrow_mut() = target_frame;
        *self.start_frame.borrow_mut() = current_frame;
        *self.animation_start.borrow_mut() = Some(Instant::now());
        *self.animation_duration.borrow_mut() = duration;
        *self.animating.borrow_mut() = true;
        *self.visible.borrow_mut() = true;

        if let Err(err) = self.cgs_window.set_shape(target_frame) {
            warn!(error=?err, "failed to set border window shape");
        }
        self.root_layer.setFrame(CGRect::new(
            CGPoint::new(0.0, 0.0),
            CGSize::new(target_frame.size.width, target_frame.size.height),
        ));

        if let Err(err) = self.cgs_window.set_level(NSPopUpMenuWindowLevel as i32) {
            warn!(error=?err, "failed to set border window level");
        }

        if let Err(err) = self.cgs_window.order_above(None) {
            warn!(error=?err, "failed to order border window above");
        }

        self.update_layer();
        self.animate_frame();
    }

    fn animate_frame(&self) {
        let start_time = match *self.animation_start.borrow() {
            Some(t) => t,
            None => return,
        };

        let duration = *self.animation_duration.borrow();
        let elapsed = start_time.elapsed();

        if elapsed >= duration {
            let target = *self.target_frame.borrow();
            *self.frame.borrow_mut() = target;
            *self.animating.borrow_mut() = false;
            self.update_layer();
            self.present();
            return;
        }

        let t = elapsed.as_secs_f64() / duration.as_secs_f64();
        let start = *self.start_frame.borrow();
        let target = *self.target_frame.borrow();
        let interpolated = interpolate_frame(start, target, t);

        *self.frame.borrow_mut() = interpolated;
        self.update_layer();
        self.present();
    }

    pub fn tick_animation(&self) -> bool {
        if !*self.animating.borrow() {
            return false;
        }
        self.animate_frame();
        *self.animating.borrow()
    }

    pub fn hide(&self) {
        *self.visible.borrow_mut() = false;
        if let Err(err) = self.cgs_window.order_out() {
            warn!(error=?err, "failed to order border window out");
        }
    }

    pub fn is_visible(&self) -> bool {
        *self.visible.borrow()
    }

    pub fn current_frame(&self) -> CGRect {
        *self.frame.borrow()
    }

    fn update_layer(&self) {
        let config = *self.config.borrow();

        CATransaction::begin();
        CATransaction::setDisableActions(true);

        let border_layer = CALayer::layer();
        let half_width = config.width / 2.0;
        let bounds = CGRect::new(
            CGPoint::new(half_width, half_width),
            CGSize::new(
                self.frame.borrow().size.width - config.width,
                self.frame.borrow().size.height - config.width,
            ),
        );
        border_layer.setFrame(CGRect::new(
            CGPoint::new(0.0, 0.0),
            CGSize::new(self.frame.borrow().size.width, self.frame.borrow().size.height),
        ));

        let color = config.color.to_nscolor();
        border_layer.setBorderColor(Some(&color.CGColor()));
        border_layer.setBorderWidth(config.width);
        border_layer.setBackgroundColor(None);

        if config.roundness > 0.0 {
            let effective_radius =
                config.roundness.min(bounds.size.width / 2.0).min(bounds.size.height / 2.0);
            border_layer.setCornerRadius(effective_radius);
        }

        unsafe {
            self.root_layer.setSublayers(None);
        }
        self.root_layer.addSublayer(&border_layer);

        CATransaction::commit();
    }

    fn present(&self) {
        let frame = *self.frame.borrow();
        let ctx: *mut CGContext =
            unsafe { SLWindowContextCreate(*G_CONNECTION, self.cgs_window.id(), ptr::null_mut()) };
        if ctx.is_null() {
            return;
        }

        unsafe {
            let clear = CGRect::new(CGPoint::new(0.0, 0.0), frame.size);
            CGContextClearRect(ctx, clear);
            CGContextSaveGState(ctx);
            CGContextTranslateCTM(ctx, 0.0, frame.size.height);
            CGContextScaleCTM(ctx, 1.0, -1.0);
            self.root_layer.renderInContext(&*ctx);
            CGContextRestoreGState(ctx);
            CGContextFlush(ctx);
            SLSFlushWindowContentRegion(*G_CONNECTION, self.cgs_window.id(), ptr::null_mut());
            CFRelease(ctx as *mut CFType);
        }
    }
}

impl Default for FocusBorderWindow {
    fn default() -> Self {
        Self::new().expect("failed to create focus border window")
    }
}
