use crate::types::BasisPoints;

pub fn to_bp(score: f64) -> BasisPoints {
    (score.clamp(0.0, 1.0) * 10_000.0).round() as BasisPoints
}

pub fn from_bp(score: BasisPoints) -> f64 {
    score as f64 / 10_000.0
}

pub fn clamp_bp(score: i64) -> BasisPoints {
    score.clamp(0, 10_000) as BasisPoints
}

pub fn ema_update(previous: BasisPoints, incoming: BasisPoints, alpha: f64) -> BasisPoints {
    let next = alpha * incoming as f64 + (1.0 - alpha) * previous as f64;
    clamp_bp(next.round() as i64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basis_points_round_trip_is_stable() {
        let input = 0.735;
        let bp = to_bp(input);
        let round_trip = from_bp(bp);
        assert_eq!(bp, 7350);
        assert!((round_trip - 0.735).abs() < 0.0001);
    }

    #[test]
    fn ema_update_blends_scores() {
        assert_eq!(ema_update(4000, 8000, 0.5), 6000);
    }
}
