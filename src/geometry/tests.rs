// Copyright 2017 Dasein Phaos aka. Luxko
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// tests

#[cfg(test)]
mod bbox {
    use geometry::bbox::*;
    use geometry::prelude::*;

    #[test]
    fn test_bbox2_new() {
        let bbox = BBox2::new(Point2::new(1, 0), Point2::new(0, 1));
        assert!(bbox.pmin == Point2::new(0, 0));
        assert!(bbox.pmax == Point2::new(1, 1));
    }

    #[test]
    fn test_bbox2_corner() {
        let bbox = BBox2::new(Point2::new(1, 0), Point2::new(0, 1));
        assert!(bbox.corner(0) == Point2::new(0, 0));
        assert!(bbox.corner(1) == Point2::new(1, 0));
        assert!(bbox.corner(2) == Point2::new(0, 1));
        assert!(bbox.corner(3) == Point2::new(1, 1));
    }

    #[test]
    fn test_bbox2_extend_contain() {
        let bbox = BBox2::new(Point2::new(1, 0), Point2::new(0, 1));
        let bbox1 = bbox.extend(Point2::new(2, 3));
        assert!(bbox1.contain(Point2::new(2, 2)));
        assert!(!bbox1.contain_lb(Point2::new(2, 2)));
    }

    #[test]
    fn test_bbox2_union() {
        let bbox = BBox2::new(Point2::new(1, 0), Point2::new(0, 1));
        let bbox1 = BBox2::new(Point2::new(20, 4), Point2::new(10, 8));
        assert!(bbox1.union(& bbox) == BBox2::new(Point2::new(0, 0), Point2::new(20, 8)));
    }

    #[test]
    fn test_bbox2_intersect() {
        let bbox: BBox2<usize> = BBox2::new(Point2::new(1, 0), Point2::new(0, 1));
        let bbox1: BBox2<usize> = BBox2::new(Point2::new(20, 4), Point2::new(10, 8));
        assert!(bbox1.intersect(&bbox)==None);
        let bbox = BBox2::new(Point2::new(0, 0), Point2::new(2, 2));
        let bbox1 = BBox2::new(Point2::new(1, 3), Point2::new(3, 1));
        let bbox2 = BBox2::new(Point2::new(1, 1), Point2::new(2, 2));
        assert!(bbox.intersect(&bbox1) == bbox2);
    }

    #[test]
    fn test_bbox2_overlap() {
        let bbox = BBox2::new(Point2::new(1, 0), Point2::new(0, 1));
        let bbox1 = BBox2::new(Point2::new(20, 4), Point2::new(10, 8));
        assert!(!bbox.overlap(&bbox1));
        let bbox2 = BBox2::new(Point2::new(-1, 4), Point2::new(17, 3));
        assert!(bbox1.overlap(&bbox2));
    }

    #[test]
    fn test_bbox2_expand_by() {
        let bbox = BBox2::new(Point2::new(1, 0), Point2::new(0, 1));
        let bbox1 = bbox.expand_by(3);
        assert!(bbox1.pmin == Point2::new(-3, -3));
        assert!(bbox1.pmax == Point2::new(4, 4));
    }

    #[test]
    fn test_bbox2_surface_area() {
        let bbox = BBox2::new(Point2::new(1, 0), Point2::new(0, 1));
        let bbox1 = bbox.expand_by(3);
        assert!(bbox1.surface_area() == 49);
    }

    #[test]
    fn test_bbox2_max_extent() {
        let bbox = BBox2::new(Point2::new(1, 0), Point2::new(0, 10));
        assert!(bbox.max_extent() == 1);
        let bbox = BBox2::new(Point2::new(100, 0), Point2::new(0, 10));
        assert!(bbox.max_extent() == 0);
    }

    #[test]
    fn test_bbox2_lerp() {
        let bbox = BBox2::new(Point2f::new(1.0 as Float, 0.0 as Float), Point2::new(0.0 as Float, 1.0 as Float));
        let p = bbox.lerp(Vector2::new(0.5 as Float, 0.7 as Float));
        assert_eq!(p, Point2f::new(0.5 as Float, 0.7 as Float));
    }

    #[test]
    fn test_bbox2_iter() {
        let bbox = BBox2::new(Point2::new(0, 0), Point2::new(2, 2));
        let mut bboxiter = bbox.into_iter();
        assert_eq!(bboxiter.next(), Some(Point2::new(0, 0)));
        assert_eq!(bboxiter.next(), Some(Point2::new(1, 0)));
        assert_eq!(bboxiter.next(), Some(Point2::new(0, 1)));
        assert_eq!(bboxiter.next(), Some(Point2::new(1, 1)));
        assert_eq!(bboxiter.next(), None);
    }
}