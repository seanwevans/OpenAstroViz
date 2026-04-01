//! Core orbital mechanics for OpenAstroViz.
//!
//! This crate exposes a Vallado-compliant SGP4 propagator so that all
//! backends share an identical reference implementation.
use sgp4::{self, chrono::NaiveDateTime, DatetimeToMinutesSinceEpochError, MinutesSinceEpoch};
use thiserror::Error;

/// Position (km) and velocity (km/s) expressed in the TEME frame.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StateVector {
    pub position_km: [f64; 3],
    pub velocity_km_s: [f64; 3],
}

/// Errors returned by the SGP4 propagator wrapper.
#[derive(Debug, Error)]
pub enum PropagationError {
    #[error("failed to parse TLE: {0}")]
    Tle(#[from] sgp4::TleError),
    #[error("failed to construct SGP4 constants: {0}")]
    Elements(#[from] sgp4::ElementsError),
    #[error("failed to convert datetime to minutes since epoch: {0:?}")]
    EpochConversion(#[from] DatetimeToMinutesSinceEpochError),
    #[error("propagation error: {0}")]
    Propagation(#[from] sgp4::Error),
}

/// Errors surfaced by GPU backend implementations.
#[derive(Debug, Error)]
pub enum GpuBackendError {
    #[error("backend is not ready")]
    NotReady,
    #[error("invalid dispatch batch size: {0}")]
    InvalidBatchSize(u32),
    #[error("shader compilation failed: {0}")]
    ShaderCompile(String),
    #[error("compute dispatch failed: {0}")]
    Dispatch(String),
}

/// Contract shared by CUDA and WebGPU compute backends.
pub trait GpuBackend {
    /// User-visible backend name.
    fn name(&self) -> &'static str;

    /// Whether this backend has been initialized and can accept work.
    fn is_ready(&self) -> bool;

    /// Dispatch one FP32 SGP4 kernel step for the given orbital batch size.
    fn dispatch_sgp4_fp32_step(&self, batch_size: u32) -> Result<(), GpuBackendError>;
}

/// Thin wrapper around the Vallado SGP4 implementation from the [`sgp4`] crate.
#[derive(Debug, Clone)]
pub struct Sgp4Propagator {
    constants: sgp4::Constants,
    elements: sgp4::Elements,
}

impl Sgp4Propagator {
    /// Create a propagator from a pair of TLE lines.
    pub fn from_tle(
        object_name: Option<String>,
        line1: &str,
        line2: &str,
    ) -> Result<Self, PropagationError> {
        let elements = sgp4::Elements::from_tle(object_name, line1.as_bytes(), line2.as_bytes())?;
        let constants = sgp4::Constants::from_elements(&elements)?;
        Ok(Self {
            constants,
            elements,
        })
    }

    /// Propagate a state by minutes since the TLE epoch.
    pub fn propagate_minutes(
        &self,
        minutes_since_epoch: f64,
    ) -> Result<StateVector, PropagationError> {
        let prediction = self
            .constants
            .propagate(MinutesSinceEpoch(minutes_since_epoch))?;
        Ok(StateVector {
            position_km: prediction.position,
            velocity_km_s: prediction.velocity,
        })
    }

    /// Propagate to an absolute UTC timestamp expressed as `NaiveDateTime`.
    pub fn propagate_datetime(
        &self,
        datetime: NaiveDateTime,
    ) -> Result<StateVector, PropagationError> {
        let minutes = self.elements.datetime_to_minutes_since_epoch(&datetime)?;
        self.propagate_minutes(minutes.0)
    }

    /// Epoch of the source TLE as a naive UTC datetime.
    pub fn epoch(&self) -> NaiveDateTime {
        self.elements.datetime
    }
}
