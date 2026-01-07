use std::cell::RefCell;
use std::ptr;
use std::time::{Duration, Instant};

use objc2_app_kit::NSPopUpMenuWindowLevel;
use objc2_core_foundation::{CFType, CGPoint, CGRect, CGSize};
use objc2_core_graphics::CGContext;
use tracing::warn;

use crate::sys::cgs_window::{CgsWindow, CgsWindowError};
use crate::sys::skylight::{
    CFRelease, G_CONNECTION, SLSDisableUpdate, SLSFlushWindowContentRegion, SLSReenableUpdate,
    SLSTransactionCommit, SLSTransactionCreate, SLSTransactionOrderWindow, SLWindowContextCreate,
};

// Core Graphics drawing functions
unsafe extern "C" {
    fn CGContextFlush(ctx: *mut CGContext);
    fn CGContextClearRect(ctx: *mut CGContext, rect: CGRect);
    fn CGContextSaveGState(ctx: *mut CGContext);
    fn CGContextRestoreGState(ctx: *mut CGContext);
    fn CGContextSetLineWidth(ctx: *mut CGContext, width: f64);
    fn CGContextSetRGBStrokeColor(ctx: *mut CGContext, r: f64, g: f64, b: f64, a: f64);
    fn CGContextStrokeRectWithWidth(ctx: *mut CGContext, rect: CGRect, width: f64);
    fn CGContextAddPath(ctx: *mut CGContext, path: *mut CFType);
    fn CGContextStrokePath(ctx: *mut CGContext);
    fn CGPathCreateWithRoundedRect(
        rect: CGRect,
        corner_width: f64,
        corner_height: f64,
        transform: *const CGAffineTransform,
    ) -> *mut CFType;
    fn CGPathRelease(path: *mut CFType);
}

/// CGAffineTransform for path creation
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CGAffineTransform {
    pub a: f64,
    pub b: f64,
    pub c: f64,
    pub d: f64,
    pub tx: f64,
    pub ty: f64,
}

impl Default for CGAffineTransform {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl CGAffineTransform {
    pub const IDENTITY: Self = Self {
        a: 1.0,
        b: 0.0,
        c: 0.0,
        d: 1.0,
        tx: 0.0,
        ty: 0.0,
    };
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BorderColor {
    pub r: f64,
    pub g: f64,
    pub b: f64,
    pub a: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
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

/// Compare two CGRect frames for equality with tolerance
fn frames_equal(a: &CGRect, b: &CGRect) -> bool {
    const EPSILON: f64 = 0.001;
    (a.origin.x - b.origin.x).abs() < EPSILON
        && (a.origin.y - b.origin.y).abs() < EPSILON
        && (a.size.width - b.size.width).abs() < EPSILON
        && (a.size.height - b.size.height).abs() < EPSILON
}

/// Focus border window using direct CGContext drawing (no CALayer overhead)
pub struct FocusBorderWindow {
    frame: RefCell<CGRect>,
    target_frame: RefCell<CGRect>,
    start_frame: RefCell<CGRect>,
    animation_start: RefCell<Option<Instant>>,
    animation_duration: RefCell<Duration>,
    cgs_window: CgsWindow,
    config: RefCell<BorderConfig>,
    visible: RefCell<bool>,
    animating: RefCell<bool>,
    /// Dirty tracking: true if border needs to be redrawn
    needs_redraw: RefCell<bool>,
    /// Last config that was rendered
    last_rendered_config: RefCell<Option<BorderConfig>>,
    /// Last frame size that was rendered
    last_rendered_size: RefCell<CGSize>,
    /// Optional corner radius from target window (for matching)
    target_corner_radius: RefCell<Option<f64>>,
}

impl FocusBorderWindow {
    pub fn new() -> Result<Self, CgsWindowError> {
        let frame = CGRect::new(CGPoint::new(0.0, 0.0), CGSize::new(100.0, 100.0));

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
            cgs_window,
            config: RefCell::new(BorderConfig::default()),
            visible: RefCell::new(false),
            animating: RefCell::new(false),
            needs_redraw: RefCell::new(true),
            last_rendered_config: RefCell::new(None),
            last_rendered_size: RefCell::new(CGSize::new(0.0, 0.0)),
            target_corner_radius: RefCell::new(None),
        })
    }

    /// Set the target window's corner radius for matching
    pub fn set_target_corner_radius(&self, radius: Option<f64>) {
        let current = *self.target_corner_radius.borrow();
        if current != radius {
            *self.target_corner_radius.borrow_mut() = radius;
            *self.needs_redraw.borrow_mut() = true;
        }
    }

    /// Get the effective corner radius (target window's radius or config roundness)
    fn effective_corner_radius(&self) -> f64 {
        let config = *self.config.borrow();
        // Use target window's corner radius if available, otherwise use config
        self.target_corner_radius.borrow().unwrap_or(config.roundness)
    }

    pub fn set_config(&self, config: BorderConfig) {
        let current_config = *self.config.borrow();
        if current_config != config {
            *self.config.borrow_mut() = config;
            *self.needs_redraw.borrow_mut() = true;
            self.draw_if_needed();
        }
    }

    pub fn show(&self, rect: CGRect) {
        let was_visible = *self.visible.borrow();
        let current_frame = *self.frame.borrow();
        let frame_changed = !frames_equal(&current_frame, &rect);

        *self.target_frame.borrow_mut() = rect;
        *self.visible.borrow_mut() = true;

        if frame_changed {
            // Use SLSDisableUpdate to prevent flicker during frame changes
            unsafe {
                SLSDisableUpdate(*G_CONNECTION);
            }

            if let Err(err) = self.cgs_window.set_shape(rect) {
                warn!(error=?err, "failed to set border window shape");
            }

            *self.frame.borrow_mut() = rect;
            *self.needs_redraw.borrow_mut() = true;

            unsafe {
                SLSReenableUpdate(*G_CONNECTION);
            }
        }

        if (!was_visible || frame_changed)
            && let Err(err) = self.cgs_window.set_level(NSPopUpMenuWindowLevel as i32)
        {
            warn!(error=?err, "failed to set border window level");
        }

        self.draw_if_needed();

        // Use SLSTransaction for batched ordering operation
        self.order_window_with_transaction();
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

        // Use SLSDisableUpdate for shape change
        unsafe {
            SLSDisableUpdate(*G_CONNECTION);
        }

        if let Err(err) = self.cgs_window.set_shape(target_frame) {
            warn!(error=?err, "failed to set border window shape");
        }

        unsafe {
            SLSReenableUpdate(*G_CONNECTION);
        }

        if let Err(err) = self.cgs_window.set_level(NSPopUpMenuWindowLevel as i32) {
            warn!(error=?err, "failed to set border window level");
        }

        // Use transaction for ordering
        self.order_window_with_transaction();

        *self.needs_redraw.borrow_mut() = true;
        self.animate_frame();
    }

    /// Order window using SLSTransaction for atomic operation
    fn order_window_with_transaction(&self) {
        unsafe {
            let transaction = SLSTransactionCreate(*G_CONNECTION);
            if !transaction.is_null() {
                // Order above (1) with no relative window (0)
                let _ = SLSTransactionOrderWindow(transaction, self.cgs_window.id(), 1, 0);
                let _ = SLSTransactionCommit(transaction, 0);
                CFRelease(transaction);
            } else {
                // Fallback to non-transaction ordering
                if let Err(err) = self.cgs_window.order_above(None) {
                    warn!(error=?err, "failed to order border window above");
                }
            }
        }
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
            *self.needs_redraw.borrow_mut() = true;
            self.draw_if_needed();
            return;
        }

        let t = elapsed.as_secs_f64() / duration.as_secs_f64();
        let start = *self.start_frame.borrow();
        let target = *self.target_frame.borrow();
        let interpolated = interpolate_frame(start, target, t);

        *self.frame.borrow_mut() = interpolated;
        *self.needs_redraw.borrow_mut() = true;
        self.draw_if_needed();
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

    /// Draw the border only if needed (config, size, or corner radius changed)
    fn draw_if_needed(&self) {
        let config = *self.config.borrow();
        let frame_size = self.frame.borrow().size;
        let last_config = *self.last_rendered_config.borrow();
        let last_size = *self.last_rendered_size.borrow();

        // Check if we need to redraw
        let config_changed = last_config != Some(config);
        let size_changed = (last_size.width - frame_size.width).abs() > 0.001
            || (last_size.height - frame_size.height).abs() > 0.001;

        if !config_changed && !size_changed && !*self.needs_redraw.borrow() {
            return;
        }

        self.draw_border();

        // Update tracking state
        *self.last_rendered_config.borrow_mut() = Some(config);
        *self.last_rendered_size.borrow_mut() = frame_size;
        *self.needs_redraw.borrow_mut() = false;
    }

    /// Draw the border directly using Core Graphics (no CALayer overhead)
    fn draw_border(&self) {
        let frame = *self.frame.borrow();
        let config = *self.config.borrow();
        let corner_radius = self.effective_corner_radius();

        let ctx: *mut CGContext =
            unsafe { SLWindowContextCreate(*G_CONNECTION, self.cgs_window.id(), ptr::null_mut()) };
        if ctx.is_null() {
            return;
        }

        unsafe {
            // Clear the context
            let clear_rect = CGRect::new(CGPoint::new(0.0, 0.0), frame.size);
            CGContextClearRect(ctx, clear_rect);

            // Save state
            CGContextSaveGState(ctx);

            // Set stroke color and line width
            CGContextSetRGBStrokeColor(
                ctx,
                config.color.r,
                config.color.g,
                config.color.b,
                config.color.a,
            );
            CGContextSetLineWidth(ctx, config.width);

            // Calculate the border rect (inset by half the line width for proper stroke)
            let half_width = config.width / 2.0;
            let border_rect = CGRect::new(
                CGPoint::new(half_width, half_width),
                CGSize::new(frame.size.width - config.width, frame.size.height - config.width),
            );

            if corner_radius > 0.0 {
                // Draw rounded rectangle
                let effective_radius = corner_radius
                    .min(border_rect.size.width / 2.0)
                    .min(border_rect.size.height / 2.0);

                let path = CGPathCreateWithRoundedRect(
                    border_rect,
                    effective_radius,
                    effective_radius,
                    ptr::null(),
                );
                if !path.is_null() {
                    CGContextAddPath(ctx, path);
                    CGContextStrokePath(ctx);
                    CGPathRelease(path);
                }
            } else {
                // Draw square rectangle
                CGContextStrokeRectWithWidth(ctx, border_rect, config.width);
            }

            // Restore state and flush
            CGContextRestoreGState(ctx);
            CGContextFlush(ctx);
            SLSFlushWindowContentRegion(*G_CONNECTION, self.cgs_window.id(), ptr::null_mut());

            // Release the context
            CFRelease(ctx as *mut CFType);
        }
    }
}

impl Default for FocusBorderWindow {
    fn default() -> Self {
        Self::new().expect("failed to create focus border window")
    }
}
