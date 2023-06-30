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
use crate::lua;
use log::info;
use mlua::{prelude::*, Variadic};

pub fn try_tostring(value: mlua::Value<'_>) -> LuaResult<String> {
    let lua = lua!();
    let tostring = lua.globals().get::<_, mlua::Function<'_>>("tostring")?;
    tostring.call::<_, String>(value)
}
pub fn tostring(value: mlua::Value<'_>) -> String {
    try_tostring(value).unwrap()
}

#[allow(clippy::panic_in_result_fn)]
pub fn print(_: &Lua, vargs: Variadic<mlua::Value<'_>>) -> LuaResult<()> {
    let lua = lua!();

    let string_table = lua.globals().get::<_, mlua::Table<'_>>("string")?;
    let format_function = string_table.get::<_, mlua::Function<'_>>("format")?;

    let mut value = vargs.into_iter().collect::<Vec<mlua::Value<'_>>>();
    let formatstring = match value.remove(0) {
        mlua::Value::String(s) => s,
        _ => unreachable!(),
    }
    .to_string_lossy()
    .to_string();
    let vargs = mlua::Variadic::from_iter(value);

    let format = format_function.call::<_, String>((formatstring, vargs))?;

    info!(target: "luminol::plugin::debug", "{}", format);
    Ok(())
}

pub fn bind() -> LuaResult<()> {
    let lua = lua!();
    let global_table = lua.globals();

    global_table.set("print", lua.create_function(print)?)?;

    Ok(())
}
