use std::f32::consts::PI;
use std::f32::consts::TAU;

pub fn average(average: f64, range: f64, value: f64) -> f64 {
    return (average * range + value) / (range + 1.0);
}

pub fn angle(x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
    return (y1 - y2).atan2(x1 - x2);
}

pub fn angle_difference(a: f32, b: f32) -> f32 {
    let difference = (b - a) % TAU;

    if difference > PI {
        return difference - TAU;
    } else if difference < -PI {
        return difference + TAU;
    } else {
        return difference;
    }
}

pub fn distance_squared(x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
    let distance_x = x1 - x2;
    let distance_y = y1 - y2;
    return distance_x * distance_x + distance_y * distance_y;
}

pub fn are_closer_than(x1: f32, y1: f32, x2: f32, y2: f32, distance: f32) -> bool {
    return distance_squared(x1, y1, x2, y2) < distance * distance;
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_abs_diff_eq;

    #[test]
    fn test_angle_difference() {
        // No difference, same values
        assert_abs_diff_eq!(0.0, angle_difference(0.0, 0.0));
        assert_abs_diff_eq!(0.0, angle_difference(1.0, 1.0));
        assert_abs_diff_eq!(0.0, angle_difference(-1.0, -1.0));
        assert_abs_diff_eq!(0.0, angle_difference(7.0, 7.0));
        assert_abs_diff_eq!(0.0, angle_difference(-7.0, -7.0));

        // No difference, different values
        assert_abs_diff_eq!(0.0, angle_difference(0.0, TAU));
        assert_abs_diff_eq!(0.0, angle_difference(0.0, -TAU));
        assert_abs_diff_eq!(0.0, angle_difference(TAU, 0.0));
        assert_abs_diff_eq!(0.0, angle_difference(-TAU, 0.0));

        // Simple difference
        assert_abs_diff_eq!(PI, angle_difference(0.0, PI));
        assert_abs_diff_eq!(-PI, angle_difference(PI, 0.0));
        assert_abs_diff_eq!(-PI, angle_difference(0.0, -PI));
        assert_abs_diff_eq!(PI, angle_difference(-PI, 0.0));

        // More complex difference
        let third = TAU / 3.0;
        assert_abs_diff_eq!(-third, angle_difference(-third, third));
        assert_abs_diff_eq!(third, angle_difference(third, -third));
        let third_doubled = third * 2.0;
        assert_abs_diff_eq!(third, angle_difference(-third_doubled, third_doubled));
        assert_abs_diff_eq!(-third, angle_difference(third_doubled, -third_doubled));
    }
}
