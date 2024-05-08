use std::fmt;
use std::time::Duration;

/// Extend `indicatif::HumanDuration` with milliseconds
#[derive(Debug)]
pub struct HumanDuration(pub Duration);

impl fmt::Display for HumanDuration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0.as_secs_f64() > 1.0 {
            indicatif::HumanDuration(self.0).fmt(f)
        } else {
            #[allow(clippy::cast_sign_loss)]
            let t = (self.0.as_secs_f64() / 1e-3).round() as usize;

            match (f.alternate(), t) {
                (true, _) => write!(f, "{t}ms"),
                (false, 1) => write!(f, "{t} millisecond"),
                (false, _) => write!(f, "{t} milliseconds"),
            }
        }
    }
}
