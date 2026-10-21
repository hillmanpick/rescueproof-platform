use crate::error::{ApiError, ApiResult};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CreateBatch {
    pub organization_id: uuid::Uuid,
    pub title: String,
    pub category: String,
    pub quantity: i32,
    pub unit: String,
    pub weight_kg: f64,
    pub area: String,
    pub pickup_address: String,
    pub contact: String,
    pub storage_notes: String,
    pub allergens: String,
    pub pickup_start: DateTime<Utc>,
    pub pickup_end: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub safety_acknowledged: bool,
}
impl CreateBatch {
    pub fn validate(&self) -> ApiResult<()> {
        for (s, max) in [
            (&self.title, 160),
            (&self.area, 100),
            (&self.pickup_address, 500),
            (&self.contact, 200),
            (&self.storage_notes, 1000),
            (&self.allergens, 500),
        ] {
            text(s, 1, max)?;
        }
        if !["bakery", "produce", "packaged", "prepared"].contains(&self.category.as_str())
            || !["portions", "items", "boxes"].contains(&self.unit.as_str())
        {
            return Err(ApiError::bad("VALIDATION", "Invalid category or unit"));
        }
        if !(1..=100000).contains(&self.quantity)
            || !self.weight_kg.is_finite()
            || self.weight_kg < 0.001
            || self.weight_kg > 100000.0
        {
            return Err(ApiError::bad("VALIDATION", "Invalid quantity or weight"));
        }
        if self.pickup_start >= self.pickup_end
            || self.pickup_end > self.expires_at
            || self.pickup_end <= Utc::now()
            || self.expires_at > Utc::now() + chrono::Duration::days(30)
        {
            return Err(ApiError::bad(
                "VALIDATION",
                "Pickup must end before expiry and within 30 days",
            ));
        }
        if !self.safety_acknowledged {
            return Err(ApiError::bad(
                "SAFETY_REQUIRED",
                "Confirm that your organisation is responsible for safe food handling",
            ));
        }
        Ok(())
    }
}
pub fn text(value: &str, min: usize, max: usize) -> ApiResult<()> {
    let len = value.trim().chars().count();
    if len < min || len > max {
        Err(ApiError::bad(
            "VALIDATION",
            format!("Text must contain {min}–{max} characters"),
        ))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_blank_and_unicode_limits() {
        assert!(text("  ", 1, 5).is_err());
        assert!(text("食品", 1, 2).is_ok());
        assert!(text("食品安全", 1, 2).is_err());
    }
}
