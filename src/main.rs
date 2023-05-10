#![warn(clippy::all, rust_2018_idioms)]
#![allow(clippy::uninlined_format_args)]
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
//
//     Additional permission under GNU GPL version 3 section 7
//
// If you modify this Program, or any covered work, by linking or combining
// it with Steamworks API by Valve Corporation, containing parts covered by
// terms of the Steamworks API by Valve Corporation, the licensors of this
// Program grant you additional permission to convey the resulting work.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

fn main() {
    #[cfg(feature = "steamworks")]
    if let Err(e) = luminol::steam::Steamworks::setup() {
        rfd::MessageDialog::new()
            .set_title("Error")
            .set_level(rfd::MessageLevel::Error)
            .set_description(&format!(
                "Steam error: {e}\nPerhaps you want to compile yourself a free copy?"
            ))
            .show();
        return;
    }

    #[cfg(debug_assertions)]
    std::thread::spawn(|| loop {
        std::thread::sleep(std::time::Duration::from_secs(5));
        let deadlocks = parking_lot::deadlock::check_deadlock();
        if deadlocks.is_empty() {
            continue;
        }

        println!("Luminol has deadlocked! Please file an issue.");
        println!("{} deadlocks detected", deadlocks.len());
        for (i, threads) in deadlocks.iter().enumerate() {
            println!("Deadlock #{}", i);
            for t in threads {
                println!("Thread Id {:#?}", t.thread_id());
                println!("{:#?}", t.backtrace());
            }
        }
        std::process::exit(1);
    });

    // Log to stdout (if you run with `RUST_LOG=debug`).
    tracing_subscriber::fmt::init();

    color_eyre::install().expect("failed to setup eyre hooks");

    #[cfg(windows)]
    if let Err(e) = setup_file_assocs() {
        eprintln!("error setting up registry {e}")
    }

    let image = image::load_from_memory(luminol::ICON).expect("Failed to load Icon data.");

    let native_options = eframe::NativeOptions {
        drag_and_drop_support: true,
        transparent: true,
        icon_data: Some(eframe::IconData {
            width: image.width(),
            height: image.height(),
            rgba: image.into_bytes(),
        }),
        wgpu_options: eframe::egui_wgpu::WgpuConfiguration {
            backends: eframe::wgpu::util::backend_bits_from_env()
                .unwrap_or(eframe::wgpu::Backends::PRIMARY),
            ..Default::default()
        },
        ..Default::default()
    };

    eframe::run_native(
        "Luminol",
        native_options,
        Box::new(|cc| Box::new(luminol::Luminol::new(cc, std::env::args_os().nth(1)))),
    )
    .expect("failed to start luminol");
}

#[cfg(windows)]
fn setup_file_assocs() -> std::io::Result<()> {
    /*
       use winreg::enums::*;
       use winreg::RegKey;

       let path = std::env::current_exe().expect("failed to get current executable path");
       let path = path.to_string_lossy();
       let command = format!("\"{path}\" \"%1\"");

       let hkcu = RegKey::predef(HKEY_CURRENT_USER);

       // RXPROJ
       let (key, _) = hkcu.create_subkey("Software\\Classes\\.rxproj")?;
       key.set_value("", &"Luminol.rxproj")?;
       let (rxproj_key, _) = hkcu.create_subkey("Software\\Classes\\Luminol.rxproj")?;
       rxproj_key.set_value("", &"RPG Maker XP Project")?;
       let (open_key, _) = rxproj_key.create_subkey("shell\\open\\command")?;
       open_key.set_value("", &command)?;
       let (icon_key, _) = rxproj_key.create_subkey("DefaultIcon")?;
       icon_key.set_value("", &format!("\"{path}\",2"))?;

       // RXDATA
       let (key, _) = hkcu.create_subkey("Software\\Classes\\.rxdata")?;
       key.set_value("", &"Luminol.rxdata")?;
       let (rxdata_key, _) = hkcu.create_subkey("Software\\Classes\\Luminol.rxdata")?;
       rxdata_key.set_value("", &"RPG Maker XP Data")?;
       let (icon_key, _) = rxdata_key.create_subkey("DefaultIcon")?;
       icon_key.set_value("", &format!("\"{path}\",3"))?;

       // LUMPROJ
       let (key, _) = hkcu.create_subkey("Software\\Classes\\.lumproj")?;
       key.set_value("", &"Luminol.lumproj")?;
       let (lumproj_key, _) = hkcu.create_subkey("Software\\Classes\\Luminol.lumproj")?;
       lumproj_key.set_value("", &"Luminol project")?;
       let (open_key, _) = lumproj_key.create_subkey("shell\\open\\command")?;
       open_key.set_value("", &command)?;
       let (icon_key, _) = lumproj_key.create_subkey("DefaultIcon")?;
       icon_key.set_value("", &format!("\"{path}\",4"))?;

       let (app_key, _) = hkcu.create_subkey("Software\\Classes\\Applications\\luminol.exe")?;
       app_key.set_value("FriendlyAppName", &"Luminol")?;
       let (supported_key, _) = app_key.create_subkey("SupportedTypes")?;
       supported_key.set_value(".rxproj", &"")?;
       supported_key.set_value(".lumproj", &"")?;
       let (open_key, _) = app_key.create_subkey("shell\\open\\command")?;
       open_key.set_value("", &command)?;
    */
    Ok(())
}
