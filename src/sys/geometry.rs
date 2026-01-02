//! converting between different binding's geometry types

use objc2_core_foundation as ic;
use serde::{Deserialize, Deserializer, Serialize};
use serde_with::{DeserializeAs, SerializeAs};

pub trait Round {
    fn round(&self) -> Self;
}

impl Round for ic::CGRect {
    fn round(&self) -> Self {
        let min_rounded = self.min().round();
        let max_rounded = self.max().round();
        ic::CGRect {
            origin: min_rounded,
            size: ic::CGSize {
                width: max_rounded.x - min_rounded.x,
                height: max_rounded.y - min_rounded.y,
            },
        }
    }
}

impl Round for ic::CGPoint {
    fn round(&self) -> Self {
        ic::CGPoint {
            x: self.x.round(),
            y: self.y.round(),
        }
    }
}

impl Round for ic::CGSize {
    fn round(&self) -> Self {
        ic::CGSize {
            width: self.width.round(),
            height: self.height.round(),
        }
    }
}

pub trait IsWithin {
    fn is_within(&self, how_much: f64, other: Self) -> bool;
}

impl IsWithin for ic::CGRect {
    fn is_within(&self, how_much: f64, other: Self) -> bool {
        self.origin.is_within(how_much, other.origin) && self.size.is_within(how_much, other.size)
    }
}

impl IsWithin for ic::CGPoint {
    fn is_within(&self, how_much: f64, other: Self) -> bool {
        self.x.is_within(how_much, other.x) && self.y.is_within(how_much, other.y)
    }
}

impl IsWithin for ic::CGSize {
    fn is_within(&self, how_much: f64, other: Self) -> bool {
        self.width.is_within(how_much, other.width) && self.height.is_within(how_much, other.height)
    }
}

impl IsWithin for f64 {
    fn is_within(&self, how_much: f64, other: Self) -> bool { (self - other).abs() < how_much }
}

pub trait SameAs: IsWithin + Sized {
    fn same_as(&self, other: Self) -> bool { self.is_within(0.1, other) }
}

impl SameAs for ic::CGRect {}
impl SameAs for ic::CGPoint {}
impl SameAs for ic::CGSize {}

pub trait CGRectExt {
    fn intersection(&self, other: &Self) -> Self;
    fn contains(&self, point: ic::CGPoint) -> bool;
    fn contains_rect(&self, other: Self) -> bool;
    fn area(&self) -> f64;
}

impl CGRectExt for ic::CGRect {
    fn intersection(&self, other: &Self) -> Self {
        let min_x = f64::max(self.min().x, other.min().x);
        let max_x = f64::min(self.max().x, other.max().x);
        let min_y = f64::max(self.min().y, other.min().y);
        let max_y = f64::min(self.max().y, other.max().y);
        ic::CGRect {
            origin: ic::CGPoint::new(min_x, min_y),
            size: ic::CGSize::new(f64::max(max_x - min_x, 0.), f64::max(max_y - min_y, 0.)),
        }
    }

    fn contains(&self, point: ic::CGPoint) -> bool {
        (self.min().x..=self.max().x).contains(&point.x)
            && (self.min().y..=self.max().y).contains(&point.y)
    }

    fn contains_rect(&self, other: Self) -> bool {
        self.min().x <= other.min().x
            && self.min().y <= other.min().y
            && self.max().x >= other.max().x
            && self.max().y >= other.max().y
    }

    fn area(&self) -> f64 { self.size.width * self.size.height }
}

#[derive(Serialize, Deserialize)]
#[serde(remote = "ic::CGRect")]
pub struct CGRectDef {
    #[serde(with = "CGPointDef")]
    pub origin: ic::CGPoint,
    #[serde(with = "CGSizeDef")]
    pub size: ic::CGSize,
}

#[derive(Serialize, Deserialize)]
#[serde(remote = "ic::CGPoint")]
pub struct CGPointDef {
    pub x: f64,
    pub y: f64,
}

#[derive(Serialize, Deserialize)]
#[serde(remote = "ic::CGSize")]
pub struct CGSizeDef {
    pub width: f64,
    pub height: f64,
}

impl SerializeAs<ic::CGRect> for CGRectDef {
    fn serialize_as<S>(value: &ic::CGRect, serializer: S) -> Result<S::Ok, S::Error>
    where S: serde::Serializer {
        CGRectDef::serialize(value, serializer)
    }
}

impl<'de> DeserializeAs<'de, ic::CGRect> for CGRectDef {
    fn deserialize_as<D>(deserializer: D) -> Result<ic::CGRect, D::Error>
    where D: Deserializer<'de> {
        CGRectDef::deserialize(deserializer)
    }
}

#[cfg(test)]
mod tests {
    use objc2_core_foundation::{CGPoint, CGRect, CGSize};

    use super::*;

    #[test]
    fn test_round_cgrect() {
        let rect = CGRect::new(CGPoint::new(10.4, 20.7), CGSize::new(100.0, 200.0));
        let rounded = rect.round();
        assert_eq!(rounded.origin.x, 10.0);
        assert_eq!(rounded.origin.y, 21.0);
        // CGRect round computes size as max - min, so 100.0 stays 100.0
        assert_eq!(rounded.size.width, 100.0);
        assert_eq!(rounded.size.height, 200.0);
    }

    #[test]
    fn test_round_cgpoint() {
        let point = CGPoint::new(10.4, 20.7);
        let rounded = point.round();
        assert_eq!(rounded.x, 10.0);
        assert_eq!(rounded.y, 21.0);
    }

    #[test]
    fn test_round_cgsize() {
        let size = CGSize::new(100.6, 200.3);
        let rounded = size.round();
        assert_eq!(rounded.width, 101.0);
        assert_eq!(rounded.height, 200.0);
    }

    #[test]
    fn test_is_within_f64() {
        let a = 10.0;
        let b = 10.05;
        assert!(a.is_within(0.1, b));
        assert!(!a.is_within(0.01, b));
    }

    #[test]
    fn test_is_within_cgpoint() {
        let a = CGPoint::new(10.0, 20.0);
        let b = CGPoint::new(10.05, 20.08);
        assert!(a.is_within(0.1, b));
        assert!(!a.is_within(0.01, b));
    }

    #[test]
    fn test_is_within_cgsize() {
        let a = CGSize::new(100.0, 200.0);
        let b = CGSize::new(100.08, 200.05);
        assert!(a.is_within(0.1, b));
        assert!(!a.is_within(0.01, b));
    }

    #[test]
    fn test_is_within_cgrect() {
        let a = CGRect::new(CGPoint::new(10.0, 20.0), CGSize::new(100.0, 200.0));
        let b = CGRect::new(CGPoint::new(10.05, 20.08), CGSize::new(100.03, 200.02));
        assert!(a.is_within(0.1, b));
        assert!(!a.is_within(0.01, b));
    }

    #[test]
    fn test_same_as_cgrect() {
        let a = CGRect::new(CGPoint::new(10.0, 20.0), CGSize::new(100.0, 200.0));
        let b = CGRect::new(CGPoint::new(10.05, 20.05), CGSize::new(100.05, 200.05));
        assert!(a.same_as(b));
    }

    #[test]
    fn test_intersection() {
        let rect1 = CGRect::new(CGPoint::new(0.0, 0.0), CGSize::new(100.0, 100.0));
        let rect2 = CGRect::new(CGPoint::new(50.0, 50.0), CGSize::new(100.0, 100.0));
        let intersection = rect1.intersection(&rect2);

        assert_eq!(intersection.origin.x, 50.0);
        assert_eq!(intersection.origin.y, 50.0);
        assert_eq!(intersection.size.width, 50.0);
        assert_eq!(intersection.size.height, 50.0);
    }

    #[test]
    fn test_no_intersection() {
        let rect1 = CGRect::new(CGPoint::new(0.0, 0.0), CGSize::new(100.0, 100.0));
        let rect2 = CGRect::new(CGPoint::new(200.0, 200.0), CGSize::new(100.0, 100.0));
        let intersection = rect1.intersection(&rect2);

        assert_eq!(intersection.size.width, 0.0);
        assert_eq!(intersection.size.height, 0.0);
    }

    #[test]
    fn test_contains_point() {
        let rect = CGRect::new(CGPoint::new(0.0, 0.0), CGSize::new(100.0, 100.0));
        assert!(rect.contains(CGPoint::new(50.0, 50.0)));
        assert!(rect.contains(CGPoint::new(0.0, 0.0)));
        assert!(rect.contains(CGPoint::new(100.0, 100.0)));
        assert!(!rect.contains(CGPoint::new(101.0, 50.0)));
        assert!(!rect.contains(CGPoint::new(-1.0, 50.0)));
    }

    #[test]
    fn test_contains_rect() {
        let rect = CGRect::new(CGPoint::new(0.0, 0.0), CGSize::new(100.0, 100.0));
        let inner = CGRect::new(CGPoint::new(10.0, 10.0), CGSize::new(80.0, 80.0));
        assert!(rect.contains_rect(inner));

        let outer = CGRect::new(CGPoint::new(-10.0, -10.0), CGSize::new(120.0, 120.0));
        assert!(!rect.contains_rect(outer));
    }

    #[test]
    fn test_area() {
        let rect = CGRect::new(CGPoint::new(0.0, 0.0), CGSize::new(100.0, 200.0));
        assert_eq!(rect.area(), 20000.0);
    }
}
