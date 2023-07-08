// Copyright (C) 2023 Lily Lyons
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
use i18n_embed::{
    fluent::{fluent_language_loader, FluentLanguageLoader},
    DesktopLanguageRequester, I18nEmbedError,
};
use once_cell::sync::Lazy;
use rust_embed::RustEmbed;

#[derive(Debug, RustEmbed)]
#[folder = "lang"]
pub struct Localizations;

static LOADER: Lazy<FluentLanguageLoader> = Lazy::new(|| {
    let loader = fluent_language_loader!();
    let requested_languages = DesktopLanguageRequester::requested_languages();
    let _ = i18n_embed::select(&loader, &Localizations, &requested_languages);

    loader
});
pub fn language_loader() -> &'static FluentLanguageLoader {
    &LOADER
}
pub fn set_language<IsoCode: ToString>(code: IsoCode) -> Result<(), I18nEmbedError> {
    i18n_embed::select(
        language_loader(),
        &Localizations,
        &[code.to_string().parse().unwrap()],
    )?;
    Ok(())
}

#[macro_export]
macro_rules! fl {
	($message_id:literal) => {
		i18n_embed_fl::fl!($crate::i18n::language_loader(), $message_id)
	};
	($message_id:literal, $($arg:tt)*) => {
		i18n_embed_fl::fl!($crate::i18n::language_loader(), $message_id, $($arg)*)
	}
}
