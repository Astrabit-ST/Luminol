// Copyright (C) 2024 Melody Madeline Lyons
//
// This file is part of Luminol.
//
// Luminol is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Luminol is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Luminol.  If not, see <http://www.gnu.org/licenses/>.
//
//     Additional permission under GNU GPL version 3 section 7
//
// If you modify this Program, or any covered work, by linking or combining
// it with Steamworks API by Valve Corporation, containing parts covered by
// terms of the Steamworks API by Valve Corporation, the licensors of this
// Program grant you additional permission to convey the resulting work.

use std::io::Write;

use luminol_config::DataFormat;
use luminol_filesystem::OpenFlags;
#[derive(Clone, Copy)]
pub struct Handler {
    format: luminol_config::DataFormat,
}

impl Handler {
    pub fn new(format: luminol_config::DataFormat) -> Self {
        Self { format }
    }

    pub fn path_for(self, filename: impl AsRef<camino::Utf8Path>) -> camino::Utf8PathBuf {
        camino::Utf8Path::new("Data")
            .join(filename)
            .with_extension(self.format.extension())
    }

    pub fn read_data<T>(
        self,
        filesystem: &impl luminol_filesystem::FileSystem,
        filename: impl AsRef<camino::Utf8Path>,
    ) -> color_eyre::Result<T>
    where
        T: for<'de> alox_48::Deserialize<'de>,
        T: ::serde::de::DeserializeOwned,
    {
        let data = filesystem.read(self.path_for(filename))?;

        match self.format {
            DataFormat::Marshal => {
                let mut de = alox_48::Deserializer::new(&data)?;
                let result = alox_48::path_to_error::deserialize(&mut de);

                result.map_err(|(error, trace)| format_traced_error(error, trace))
            }
            DataFormat::Ron { .. } => {
                let mut de = ron::de::Deserializer::from_bytes(&data)?;
                serde_path_to_error::deserialize(&mut de).map_err(format_path_to_error)
            }
            DataFormat::Json { .. } => {
                let mut de = serde_json::de::Deserializer::from_slice(&data);
                serde_path_to_error::deserialize(&mut de).map_err(format_path_to_error)
            }
        }
    }

    pub fn read_data_from<T>(self, data: &[u8]) -> color_eyre::Result<T>
    where
        T: for<'de> alox_48::Deserialize<'de>,
        T: ::serde::de::DeserializeOwned,
    {
        match self.format {
            DataFormat::Marshal => {
                let mut de = alox_48::Deserializer::new(data)?;
                let result = alox_48::path_to_error::deserialize(&mut de);

                result.map_err(|(error, trace)| format_traced_error(error, trace))
            }
            DataFormat::Ron { .. } => {
                let mut de = ron::de::Deserializer::from_bytes(data)?;
                serde_path_to_error::deserialize(&mut de).map_err(format_path_to_error)
            }
            DataFormat::Json { .. } => {
                let mut de = serde_json::de::Deserializer::from_slice(data);
                serde_path_to_error::deserialize(&mut de).map_err(format_path_to_error)
            }
        }
    }

    pub fn write_data<T>(
        self,
        data: &T,
        filesystem: &impl luminol_filesystem::FileSystem,
        filename: impl AsRef<camino::Utf8Path>,
    ) -> color_eyre::Result<()>
    where
        T: ::serde::Serialize,
        T: alox_48::Serialize,
    {
        let mut file = filesystem.open_file(
            self.path_for(filename),
            OpenFlags::Create | OpenFlags::Truncate | OpenFlags::Write,
        )?;

        match self.format {
            DataFormat::Marshal => {
                let mut serializer = alox_48::Serializer::new();
                alox_48::path_to_error::serialize(data, &mut serializer)
                    .map_err(|(error, trace)| format_traced_error(error, trace))?;
                file.write_all(&serializer.output)?;
            }
            DataFormat::Ron { pretty } => {
                let config = pretty.then(|| ron::ser::PrettyConfig::new().struct_names(true));
                let mut ser =
                    ron::Serializer::with_options(&mut file, config, ron::Options::default())?;
                serde_path_to_error::serialize(data, &mut ser)?;
            }
            DataFormat::Json { pretty } => {
                if pretty {
                    let mut ser = serde_json::Serializer::pretty(&mut file);
                    serde_path_to_error::serialize(data, &mut ser)?;
                } else {
                    let mut ser = serde_json::Serializer::new(&mut file);
                    serde_path_to_error::serialize(data, &mut ser)?;
                }
            }
        };

        Ok(())
    }

    pub fn write_data_to<T>(self, data: &T, buffer: &mut Vec<u8>) -> color_eyre::Result<()>
    where
        T: ::serde::Serialize,
        T: alox_48::Serialize,
    {
        match self.format {
            DataFormat::Marshal => {
                let mut serializer = alox_48::Serializer::new();
                alox_48::path_to_error::serialize(data, &mut serializer)
                    .map_err(|(error, trace)| format_traced_error(error, trace))?;
                buffer.extend_from_slice(&serializer.output);
            }
            DataFormat::Ron { pretty } => {
                let config = pretty.then(|| ron::ser::PrettyConfig::new().struct_names(true));
                let mut ser =
                    ron::Serializer::with_options(buffer, config, ron::Options::default())?;
                serde_path_to_error::serialize(data, &mut ser)?;
            }
            DataFormat::Json { pretty } => {
                if pretty {
                    let mut ser = serde_json::Serializer::pretty(buffer);
                    serde_path_to_error::serialize(data, &mut ser)?;
                } else {
                    let mut ser = serde_json::Serializer::new(buffer);
                    serde_path_to_error::serialize(data, &mut ser)?;
                }
            }
        };

        Ok(())
    }

    pub fn read_nil_padded<T>(
        self,
        filesystem: &impl luminol_filesystem::FileSystem,
        filename: impl AsRef<camino::Utf8Path>,
    ) -> color_eyre::Result<Vec<T>>
    where
        T: for<'de> alox_48::Deserialize<'de>,
        T: ::serde::de::DeserializeOwned,
    {
        let data = filesystem.read(self.path_for(filename))?;

        match self.format {
            DataFormat::Marshal => {
                let mut de = alox_48::Deserializer::new(&data)?;
                let mut trace = alox_48::path_to_error::Trace::default();
                let de = alox_48::path_to_error::Deserializer::new(&mut de, &mut trace);

                luminol_data::helpers::nil_padded_alox::deserialize_with(de)
                    .map_err(|error| format_traced_error(error, trace))
            }
            DataFormat::Ron { .. } => {
                let mut de = ron::de::Deserializer::from_bytes(&data)?;
                let mut track = serde_path_to_error::Track::new();
                let de = serde_path_to_error::Deserializer::new(&mut de, &mut track);

                luminol_data::helpers::nil_padded_serde::deserialize(de).map_err(|inner| {
                    let error = serde_path_to_error::Error::new(track.path(), inner);
                    format_path_to_error(error)
                })
            }
            DataFormat::Json { .. } => {
                let mut de = serde_json::de::Deserializer::from_slice(&data);
                let mut track = serde_path_to_error::Track::new();
                let de = serde_path_to_error::Deserializer::new(&mut de, &mut track);

                luminol_data::helpers::nil_padded_serde::deserialize(de).map_err(|inner| {
                    let error = serde_path_to_error::Error::new(track.path(), inner);
                    format_path_to_error(error)
                })
            }
        }
    }

    pub fn read_nil_padded_from<T>(self, data: &[u8]) -> color_eyre::Result<Vec<T>>
    where
        T: for<'de> alox_48::Deserialize<'de>,
        T: ::serde::de::DeserializeOwned,
    {
        match self.format {
            DataFormat::Marshal => {
                let mut de = alox_48::Deserializer::new(data)?;
                let mut trace = alox_48::path_to_error::Trace::default();
                let de = alox_48::path_to_error::Deserializer::new(&mut de, &mut trace);

                luminol_data::helpers::nil_padded_alox::deserialize_with(de)
                    .map_err(|error| format_traced_error(error, trace))
            }
            DataFormat::Ron { .. } => {
                let mut de = ron::de::Deserializer::from_bytes(data)?;
                let mut track = serde_path_to_error::Track::new();
                let de = serde_path_to_error::Deserializer::new(&mut de, &mut track);

                luminol_data::helpers::nil_padded_serde::deserialize(de).map_err(|inner| {
                    let error = serde_path_to_error::Error::new(track.path(), inner);
                    format_path_to_error(error)
                })
            }
            DataFormat::Json { .. } => {
                let mut de = serde_json::de::Deserializer::from_slice(data);
                let mut track = serde_path_to_error::Track::new();
                let de = serde_path_to_error::Deserializer::new(&mut de, &mut track);

                luminol_data::helpers::nil_padded_serde::deserialize(de).map_err(|inner| {
                    let error = serde_path_to_error::Error::new(track.path(), inner);
                    format_path_to_error(error)
                })
            }
        }
    }

    pub fn write_nil_padded<T>(
        self,
        data: &[T],
        filesystem: &impl luminol_filesystem::FileSystem,
        filename: impl AsRef<camino::Utf8Path>,
    ) -> color_eyre::Result<()>
    where
        T: ::serde::Serialize,
        T: alox_48::Serialize,
    {
        let mut file = filesystem.open_file(
            self.path_for(filename),
            OpenFlags::Create | OpenFlags::Truncate | OpenFlags::Write,
        )?;

        match self.format {
            DataFormat::Marshal => {
                let mut trace = alox_48::path_to_error::Trace::new();
                let mut ser = alox_48::Serializer::new();
                let trace_ser = alox_48::path_to_error::Serializer::new(&mut ser, &mut trace);

                luminol_data::helpers::nil_padded_alox::serialize_with(data, trace_ser)
                    .map_err(|error| format_traced_error(error, trace))?;
                file.write_all(&ser.output)?;
            }
            DataFormat::Json { pretty } => {
                let mut track = serde_path_to_error::Track::new();
                if pretty {
                    let mut ser = serde_json::Serializer::pretty(&mut file);
                    let ser = serde_path_to_error::Serializer::new(&mut ser, &mut track);

                    luminol_data::helpers::nil_padded_serde::serialize(data, ser).map_err(
                        |inner| {
                            let error = serde_path_to_error::Error::new(track.path(), inner);
                            format_path_to_error(error)
                        },
                    )?;
                } else {
                    let mut ser = serde_json::Serializer::new(&mut file);
                    let ser = serde_path_to_error::Serializer::new(&mut ser, &mut track);

                    luminol_data::helpers::nil_padded_serde::serialize(data, ser).map_err(
                        |inner| {
                            let error = serde_path_to_error::Error::new(track.path(), inner);
                            format_path_to_error(error)
                        },
                    )?;
                }
            }
            DataFormat::Ron { pretty } => {
                let mut track = serde_path_to_error::Track::new();
                let config = pretty.then(|| ron::ser::PrettyConfig::new().struct_names(true));
                let mut ser =
                    ron::Serializer::with_options(&mut file, config, ron::Options::default())?;
                let ser = serde_path_to_error::Serializer::new(&mut ser, &mut track);

                luminol_data::helpers::nil_padded_serde::serialize(data, ser).map_err(|inner| {
                    let error = serde_path_to_error::Error::new(track.path(), inner);
                    format_path_to_error(error)
                })?;
            }
        }

        file.flush()?;

        Ok(())
    }

    pub fn write_nil_padded_to<T>(self, data: &[T], buffer: &mut Vec<u8>) -> color_eyre::Result<()>
    where
        T: ::serde::Serialize,
        T: alox_48::Serialize,
    {
        match self.format {
            DataFormat::Marshal => {
                let mut trace = alox_48::path_to_error::Trace::new();
                let mut ser = alox_48::Serializer::new();
                let trace_ser = alox_48::path_to_error::Serializer::new(&mut ser, &mut trace);

                luminol_data::helpers::nil_padded_alox::serialize_with(data, trace_ser)
                    .map_err(|error| format_traced_error(error, trace))?;
                buffer.extend_from_slice(&ser.output);
            }
            DataFormat::Json { pretty } => {
                let mut track = serde_path_to_error::Track::new();
                if pretty {
                    let mut ser = serde_json::Serializer::pretty(buffer);
                    let ser = serde_path_to_error::Serializer::new(&mut ser, &mut track);

                    luminol_data::helpers::nil_padded_serde::serialize(data, ser).map_err(
                        |inner| {
                            let error = serde_path_to_error::Error::new(track.path(), inner);
                            format_path_to_error(error)
                        },
                    )?;
                } else {
                    let mut ser = serde_json::Serializer::new(buffer);
                    let ser = serde_path_to_error::Serializer::new(&mut ser, &mut track);

                    luminol_data::helpers::nil_padded_serde::serialize(data, ser).map_err(
                        |inner| {
                            let error = serde_path_to_error::Error::new(track.path(), inner);
                            format_path_to_error(error)
                        },
                    )?;
                }
            }
            DataFormat::Ron { pretty } => {
                let mut track = serde_path_to_error::Track::new();
                let config = pretty.then(|| ron::ser::PrettyConfig::new().struct_names(true));
                let mut ser =
                    ron::Serializer::with_options(buffer, config, ron::Options::default())?;
                let ser = serde_path_to_error::Serializer::new(&mut ser, &mut track);

                luminol_data::helpers::nil_padded_serde::serialize(data, ser).map_err(|inner| {
                    let error = serde_path_to_error::Error::new(track.path(), inner);
                    format_path_to_error(error)
                })?;
            }
        }

        Ok(())
    }

    pub fn remove_file(
        self,
        filesystem: &impl luminol_filesystem::FileSystem,
        filename: impl AsRef<camino::Utf8Path>,
    ) -> color_eyre::Result<()> {
        let path = camino::Utf8Path::new("Data")
            .join(filename)
            .with_extension(self.format.extension());
        filesystem.remove_file(path)
    }
}

pub fn format_path_to_error<E>(error: serde_path_to_error::Error<E>) -> color_eyre::Report
where
    E: serde::de::Error + Send + Sync + 'static,
{
    error.into() // TODO
}

pub fn format_traced_error(
    error: impl Into<color_eyre::Report>,
    trace: alox_48::path_to_error::Trace,
) -> color_eyre::Report {
    let mut error = error.into();
    for context in trace.context {
        error = error.wrap_err(context);
    }
    error
}
