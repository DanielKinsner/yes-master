pub(crate) type AaudioResult = i32;

pub(crate) const AAUDIO_FORMAT_PCM_FLOAT: i32 = 2;
pub(crate) const AAUDIO_ERROR_ILLEGAL_ARGUMENT: AaudioResult = -898;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ActualStreamConfig {
    pub(crate) channel_count: usize,
    pub(crate) sample_rate: u32,
    pub(crate) format: i32,
}

impl ActualStreamConfig {
    pub(crate) fn from_raw(
        channel_count: i32,
        sample_rate: i32,
        format: i32,
    ) -> Result<Self, AaudioResult> {
        if channel_count <= 0 || sample_rate <= 0 {
            return Err(AAUDIO_ERROR_ILLEGAL_ARGUMENT);
        }
        Ok(Self {
            channel_count: channel_count as usize,
            sample_rate: sample_rate as u32,
            format,
        })
    }
}

pub(crate) fn callback_buffer_samples(num_frames: i32, channel_count: usize) -> usize {
    let frames = num_frames.max(0) as usize;
    frames.saturating_mul(channel_count)
}

pub(crate) fn validate_actual_stream(
    actual: ActualStreamConfig,
    pump_channels: usize,
) -> Result<(), AaudioResult> {
    if actual.format != AAUDIO_FORMAT_PCM_FLOAT || actual.channel_count != pump_channels {
        return Err(AAUDIO_ERROR_ILLEGAL_ARGUMENT);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn callback_buffer_uses_stream_channel_count_not_source_channel_count() {
        assert_eq!(callback_buffer_samples(128, 2), 256);
        assert_eq!(callback_buffer_samples(128, 6), 768);
    }

    #[test]
    fn callback_buffer_treats_negative_frame_requests_as_empty() {
        assert_eq!(callback_buffer_samples(-1, 2), 0);
    }

    #[test]
    fn actual_stream_config_rejects_non_positive_stream_values() {
        assert_eq!(
            ActualStreamConfig::from_raw(0, 48_000, AAUDIO_FORMAT_PCM_FLOAT),
            Err(AAUDIO_ERROR_ILLEGAL_ARGUMENT),
        );
        assert_eq!(
            ActualStreamConfig::from_raw(2, 0, AAUDIO_FORMAT_PCM_FLOAT),
            Err(AAUDIO_ERROR_ILLEGAL_ARGUMENT),
        );
    }

    #[test]
    fn actual_stream_validation_refuses_channel_or_format_mismatch() {
        let stereo_float =
            ActualStreamConfig::from_raw(2, 48_000, AAUDIO_FORMAT_PCM_FLOAT).expect("config");
        assert_eq!(validate_actual_stream(stereo_float, 2), Ok(()));
        assert_eq!(
            validate_actual_stream(stereo_float, 6),
            Err(AAUDIO_ERROR_ILLEGAL_ARGUMENT),
        );

        let stereo_i16 = ActualStreamConfig::from_raw(2, 48_000, 1).expect("config");
        assert_eq!(
            validate_actual_stream(stereo_i16, 2),
            Err(AAUDIO_ERROR_ILLEGAL_ARGUMENT),
        );
    }

    #[test]
    fn actual_stream_validation_allows_resampled_streams() {
        let resampled =
            ActualStreamConfig::from_raw(2, 44_100, AAUDIO_FORMAT_PCM_FLOAT).expect("config");
        assert_eq!(validate_actual_stream(resampled, 2), Ok(()));
    }
}
