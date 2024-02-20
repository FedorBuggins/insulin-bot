use chrono::{DateTime, Utc};

pub struct SugarLevel {
  millimoles_per_liter: f32,
}

impl SugarLevel {
  #[must_use]
  pub fn from_millimoles_per_liter(
    millimoles_per_liter: f32,
  ) -> Self {
    Self {
      millimoles_per_liter,
    }
  }

  #[must_use]
  pub fn as_millimoles_per_liter(&self) -> f32 {
    self.millimoles_per_liter
  }
}

pub struct SugarMeasurement {
  pub date_time: DateTime<Utc>,
  pub level: SugarLevel,
}

impl SugarMeasurement {
  #[must_use]
  pub fn from_now(level: SugarLevel) -> Self {
    let date_time = Utc::now();
    Self { date_time, level }
  }
}
