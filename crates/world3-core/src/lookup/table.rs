/// Piecewise-linear lookup table — the fundamental building block of World 3.
/// Every non-linear relationship in the model (e.g. "mortality vs food") is
/// encoded as one of these tables, matching the original Dynamo implementation.
///
/// Outside the defined range, values are clamped to the endpoint values.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LookupTable {
    pub name: String,
    /// x values — must be strictly increasing
    pub x: Vec<f64>,
    /// y values — same length as x
    pub y: Vec<f64>,
}

impl LookupTable {
    pub fn new(name: impl Into<String>, x: Vec<f64>, y: Vec<f64>) -> Self {
        assert_eq!(x.len(), y.len(), "LookupTable x and y must have equal length");
        assert!(x.len() >= 2, "LookupTable must have at least 2 points");
        Self { name: name.into(), x, y }
    }

    /// Evaluate the table at `x_in` using piecewise linear interpolation.
    /// Values outside [x[0], x[n-1]] are clamped to the endpoint y values.
    pub fn eval(&self, x_in: f64) -> f64 {
        let x_clamped = x_in.clamp(self.x[0], *self.x.last().unwrap());

        // Binary search for the right segment
        let pos = self.x.partition_point(|&xi| xi <= x_clamped);

        if pos == 0 {
            return self.y[0];
        }
        if pos >= self.x.len() {
            return *self.y.last().unwrap();
        }

        let x0 = self.x[pos - 1];
        let x1 = self.x[pos];
        let y0 = self.y[pos - 1];
        let y1 = self.y[pos];

        // Linear interpolation
        let t = (x_clamped - x0) / (x1 - x0);
        y0 + t * (y1 - y0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lookup_basic_interpolation() {
        let t = LookupTable::new("test", vec![0.0, 1.0, 2.0], vec![0.0, 10.0, 20.0]);
        assert!((t.eval(0.5) - 5.0).abs() < 1e-9);
        assert!((t.eval(1.5) - 15.0).abs() < 1e-9);
    }

    #[test]
    fn test_lookup_clamping() {
        let t = LookupTable::new("test", vec![0.0, 1.0], vec![5.0, 10.0]);
        assert!((t.eval(-1.0) - 5.0).abs() < 1e-9);
        assert!((t.eval(2.0) - 10.0).abs() < 1e-9);
    }

    #[test]
    fn test_lookup_exact_points() {
        let t = LookupTable::new("test", vec![0.0, 1.0, 2.0], vec![3.0, 7.0, 11.0]);
        assert!((t.eval(0.0) - 3.0).abs() < 1e-9);
        assert!((t.eval(1.0) - 7.0).abs() < 1e-9);
        assert!((t.eval(2.0) - 11.0).abs() < 1e-9);
    }
}
