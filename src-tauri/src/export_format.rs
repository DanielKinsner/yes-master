//! Encoding requests are independent of mastering settings and delivered metering.
use crate::types::{CommandError, CommandResult};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "format", rename_all = "snake_case", try_from = "EncodingWire")]
pub enum ExportEncoding {
    #[default]
    Wav,
    Mp3 {
        bitrate_kbps: u16,
    },
    Flac,
    M4a {
        bitrate_kbps: u16,
    },
    Aac {
        bitrate_kbps: u16,
    },
    Ogg {
        quality: u8,
    },
    Aiff,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct EncodingWire {
    format: String,
    bitrate_kbps: Option<u16>,
    quality: Option<u8>,
}

impl TryFrom<EncodingWire> for ExportEncoding {
    type Error = String;
    fn try_from(value: EncodingWire) -> Result<Self, Self::Error> {
        let encoding = match (value.format.as_str(), value.bitrate_kbps, value.quality) {
            ("wav", None, None) => Self::Wav,
            ("flac", None, None) => Self::Flac,
            ("aiff", None, None) => Self::Aiff,
            ("mp3", Some(bitrate_kbps), None) => Self::Mp3 { bitrate_kbps },
            ("m4a", Some(bitrate_kbps), None) => Self::M4a { bitrate_kbps },
            ("aac", Some(bitrate_kbps), None) => Self::Aac { bitrate_kbps },
            ("ogg", None, Some(quality)) => Self::Ogg { quality },
            _ => return Err("Invalid codec/container/quality combination".into()),
        };
        encoding.validate().map_err(|e| e.to_string())?;
        Ok(encoding)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeliveredFormat {
    pub encoding: ExportEncoding,
    pub codec: String,
    pub container: String,
    pub sample_rate: u32,
    pub channels: u16,
    pub bit_depth: Option<u16>,
    pub requested_sample_rate: Option<u32>,
    pub requested_bit_depth: u16,
}

impl ExportEncoding {
    pub fn resolve(request: Option<Self>, legacy_mp3: Option<u16>) -> CommandResult<Self> {
        let resolved = match (request, legacy_mp3) {
            (None, None) => Self::Wav,
            (None, Some(bitrate_kbps)) => Self::Mp3 { bitrate_kbps },
            (Some(value), None) => value,
            (Some(Self::Mp3 { bitrate_kbps }), Some(legacy)) if bitrate_kbps == legacy => {
                Self::Mp3 { bitrate_kbps }
            }
            _ => {
                return Err(CommandError::Render(
                    "Conflicting export encoding requests".into(),
                ))
            }
        };
        resolved.validate()?;
        Ok(resolved)
    }

    pub fn validate(self) -> CommandResult<()> {
        match self {
            Self::Mp3 { bitrate_kbps }
            | Self::M4a { bitrate_kbps }
            | Self::Aac { bitrate_kbps }
                if !matches!(bitrate_kbps, 128 | 192 | 256 | 320) =>
            {
                Err(CommandError::Render(
                    "Bitrate must be 128, 192, 256 or 320 kbps".into(),
                ))
            }
            Self::Ogg { quality } if !matches!(quality, 4 | 6 | 8) => Err(CommandError::Render(
                "Vorbis quality must be 4, 6 or 8".into(),
            )),
            _ => Ok(()),
        }
    }

    pub fn extension(self) -> &'static str {
        match self {
            Self::Wav => "wav",
            Self::Mp3 { .. } => "mp3",
            Self::Flac => "flac",
            Self::M4a { .. } => "m4a",
            Self::Aac { .. } => "aac",
            Self::Ogg { .. } => "ogg",
            Self::Aiff => "aiff",
        }
    }

    pub fn is_lossy(self) -> bool {
        matches!(
            self,
            Self::Mp3 { .. } | Self::M4a { .. } | Self::Aac { .. } | Self::Ogg { .. }
        )
    }

    pub fn needs_sidecar(self) -> bool {
        !matches!(self, Self::Wav | Self::Mp3 { .. })
    }

    pub fn delivery_rate(self, rate: u32) -> u32 {
        match self {
            Self::Mp3 { .. } => crate::mp3::delivery_rate(rate),
            value if value.is_lossy() => {
                if rate % 44_100 == 0 {
                    44_100
                } else {
                    48_000
                }
            }
            _ => rate,
        }
    }

    pub fn delivery_bits(self, bits: u16) -> u16 {
        if self.is_lossy() {
            32
        } else if self.needs_sidecar() {
            bits.min(24)
        } else {
            bits
        }
    }

    pub fn delivered(
        self,
        rate: u32,
        channels: u16,
        bits: u16,
        requested_rate: Option<u32>,
        requested_bits: u16,
    ) -> DeliveredFormat {
        let (codec, container) = match self {
            Self::Wav => (
                if bits == 32 {
                    "pcm_f32le"
                } else if bits == 16 {
                    "pcm_s16le"
                } else {
                    "pcm_s24le"
                },
                "wav",
            ),
            Self::Mp3 { .. } => ("mp3", "mp3"),
            Self::Flac => ("flac", "flac"),
            Self::M4a { .. } => ("aac_lc", "m4a"),
            Self::Aac { .. } => ("aac_lc", "adts"),
            Self::Ogg { .. } => ("vorbis", "ogg"),
            Self::Aiff => (if bits == 16 { "pcm_s16be" } else { "pcm_s24be" }, "aiff"),
        };
        DeliveredFormat {
            encoding: self,
            codec: codec.into(),
            container: container.into(),
            sample_rate: rate,
            channels,
            bit_depth: if self.is_lossy() { None } else { Some(bits) },
            requested_sample_rate: requested_rate,
            requested_bit_depth: requested_bits,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn legacy_requests_and_conflicts() {
        assert_eq!(
            ExportEncoding::resolve(None, None).unwrap(),
            ExportEncoding::Wav
        );
        assert_eq!(
            ExportEncoding::resolve(None, Some(192)).unwrap(),
            ExportEncoding::Mp3 { bitrate_kbps: 192 }
        );
        assert!(ExportEncoding::resolve(Some(ExportEncoding::Flac), Some(320)).is_err());
        assert!(ExportEncoding::resolve(None, Some(12)).is_err());
        assert!(
            serde_json::from_str::<ExportEncoding>(r#"{"format":"flac","bitrate_kbps":256}"#)
                .is_err()
        );
    }
    #[test]
    fn explicit_container_and_precision_conversion() {
        let choice = ExportEncoding::M4a { bitrate_kbps: 256 };
        let delivered = choice.delivered(choice.delivery_rate(96_000), 2, 32, Some(96_000), 24);
        assert_eq!(delivered.bit_depth, None);
        assert_eq!(delivered.sample_rate, 48_000);
        assert_eq!(delivered.container, "m4a");
        assert_eq!(ExportEncoding::Flac.delivery_bits(32), 24);
        assert_eq!(ExportEncoding::Wav.delivery_bits(32), 32);
        assert_eq!(
            ExportEncoding::Mp3 { bitrate_kbps: 320 }.delivery_rate(32_000),
            32_000
        );
        let bytes = serde_json::to_string(&delivered).unwrap();
        assert_eq!(
            serde_json::from_str::<DeliveredFormat>(&bytes).unwrap(),
            delivered
        );
    }
}
