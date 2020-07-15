////////////////////////////////////////////////////////////////////////////////
// Atma structured color palette
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Command line dispatching.
////////////////////////////////////////////////////////////////////////////////

// Local imports.
use crate::cell::CellRef;
use crate::cell::CellSelection;
use crate::cell::CellSelector;
use crate::cell::PositionSelector;
use crate::command::AtmaOptions;
use crate::command::CommandOption;
use crate::command::ExportOption;
use crate::Config;
use crate::error::FileError;
use crate::palette::Palette;
use crate::Settings;

// External library imports.
use log::*;
use anyhow::anyhow;

// Standard library imports.
use std::path::PathBuf;
use std::path::Path;


////////////////////////////////////////////////////////////////////////////////
// Constants
////////////////////////////////////////////////////////////////////////////////

/// Error message returned when no active palette is loaded.
const NO_PALETTE: &'static str = "No active palette loaded.";


////////////////////////////////////////////////////////////////////////////////
// dispatch
////////////////////////////////////////////////////////////////////////////////
/// Executes the given `AtmaOptions` on the given `Palette`.
pub fn dispatch(
    palette: Option<Palette>,
    opts: AtmaOptions,
    config: Config,
    settings: Settings,
    cur_dir: PathBuf)
    -> Result<(), anyhow::Error>
{
    trace!("Begin command dispatch.");
    use CommandOption::*;
    use anyhow::Context as _;

    match opts.command {

        // New
        ////////////////////////////////////////////////////////////////////////
        New {
            name,
            no_history,
            no_config_file,
            no_settings_file,
            set_active,
        } => new_palette(
                palette.unwrap_or(Palette::new()
                    .with_load_path(
                        cur_dir.join(&config.default_palette_path))),
                name,
                no_history,
                if no_config_file { None } else { Some(config) },
                if no_settings_file { None } else { Some(settings) },
                set_active)
            .context("Command 'new' failed"),

        // List
        ////////////////////////////////////////////////////////////////////////
        List { selection } => {
            let pal = palette.ok_or(anyhow!(NO_PALETTE))?;

            // TODO: Move all of this to an inner function.
            debug!("Start listing for selection {:?}", selection);
            let selection = selection.unwrap_or(CellSelector::All.into());
            let index_selection = selection.resolve(pal.inner());
            debug!("Start listing for {:?}", index_selection);

            for idx in index_selection {
                if let Ok(Some(c)) = pal.inner()
                    .color(&CellRef::Index(idx))
                {
                    print!("{:4X} {:X}", idx, c);
                    if let Some(pos) = pal.inner()
                        .assigned_position(&CellRef::Index(idx))
                    {
                        print!(" {}", pos);
                    }
                    if let Some(name) = pal.inner()
                        .assigned_name(&CellRef::Index(idx))
                    {
                        print!(" \"{}\"", name);
                    }
                    println!();
                }
            }
            Ok(())
        },

        // Insert
        ////////////////////////////////////////////////////////////////////////
        Insert { exprs, name, at } => {
            let mut pal = palette.ok_or(anyhow!(NO_PALETTE))?;
            if exprs.is_empty() {
                println!("No expressions to insert.");
                return Ok(()); 
            }
            let at = at.unwrap_or(config.default_positioning);

            pal.insert_exprs(&exprs[..], name, at)
                .context("insert command failed.")?;

            pal.write_to_load_path()
                .map(|_| ())
                .context("Failed to write palette")
        },

        // Delete
        ////////////////////////////////////////////////////////////////////////
        Delete { selection } => match selection {
            Some(selection) => {
                let mut pal = palette.ok_or(anyhow!(NO_PALETTE))?;
                pal.delete_selection(selection)
                    .context("delete command failed.")?;

                pal.write_to_load_path()
                    .map(|_| ())
                    .context("Failed to write palette")
            },
            None => {
                println!("No cell selection; nothing to delete.");
                Ok(())
            },
        },

        // Move
        ////////////////////////////////////////////////////////////////////////
        Move { selection, to } => match selection {
            Some(selection) => {
                let mut pal = palette.ok_or(anyhow!(NO_PALETTE))?;
                let to = to.unwrap_or(config.default_positioning);
                pal.move_selection(selection, to)?;

                pal.write_to_load_path()
                    .map(|_| ())
                    .context("Failed to write palette")
            },
            None => {
                println!("No cell selection; nothing to move.");
                Ok(())
            },
        },

        // Set
        ////////////////////////////////////////////////////////////////////////
        Set => unimplemented!(),

        // Unset
        ////////////////////////////////////////////////////////////////////////
        Unset => unimplemented!(),

        // Undo
        ////////////////////////////////////////////////////////////////////////
        Undo { count } => {
            let mut pal = palette.ok_or(anyhow!(NO_PALETTE))?;
            let count = count.unwrap_or(1);
            if count == 0 {
                println!("0 undo operations performed.");
                return Ok(());
            };
            let performed = pal.undo(count);
            match performed {
                0 => {
                    println!("No undo operations recorded.");
                    return Ok(());
                },
                1 => println!("Undo operation completed."),
                _ => println!("{} undo operations performed.", performed),
            }
            pal.write_to_load_path()
                .map(|_| ())
                .context("Failed to write palette")
        },

        // Redo
        ////////////////////////////////////////////////////////////////////////
        Redo { count } => {
            let mut pal = palette.ok_or(anyhow!(NO_PALETTE))?;
            let count = count.unwrap_or(1);
            if count == 0 {
                println!("0 redo operations performed.");
                return Ok(());
            };
            let performed = pal.redo(count);
            match performed {
                0 => {
                    println!("No redo operations recorded.");
                    return Ok(());
                },
                1 => println!("Redo operation completed."),
                _ => println!("{} redo operations performed.", performed),
            }
            pal.write_to_load_path()
                .map(|_| ())
                .context("Failed to write palette")
        },

        // Import
        ////////////////////////////////////////////////////////////////////////
        Import => unimplemented!(),

        // Export
        ////////////////////////////////////////////////////////////////////////
        Export { export_option } => {
            let pal = palette.ok_or(anyhow!(NO_PALETTE))?;
            match export_option {
                ExportOption::Png { selection, output } => {
                    write_png(
                        &pal,
                        selection.unwrap_or(CellSelector::All.into()),
                        &cur_dir.clone().join(output))
                },
            }
        },
    }
}


/// Initializes a new palette.
fn new_palette(
    mut palette: Palette,
    name: Option<String>,
    no_history: bool,
    config: Option<Config>,
    settings: Option<Settings>,
    set_active: bool)
    -> Result<(), anyhow::Error>
{
    trace!("Build new palette.");
    use crate::error::FileErrorContext as _;

    fn already_exists(e: &FileError) -> bool {
        e.is_io_error_kind(std::io::ErrorKind::AlreadyExists)
    }

    if !no_history { palette = palette.with_history(); }
    if let Some(name) = name {
        let _ = palette.inner_mut().assign_name(name, PositionSelector::ALL)?;
    }

    if let Some(config) = config {
        let res = config.write_to_load_path_if_new();
        if res.as_ref().map_err(already_exists).err().unwrap_or(false) {
            info!("Config file already exists.");
            debug!("Config {:?}", config.load_path());
        } else {
            let _ = res.with_context(|| 
                if let Some(path) = config.load_path() {
                    format!("Error writing config file: {}", path.display())
                } else {
                    format!("Error writing config file")
                })?;
        }
    }

    if let Some(mut settings) = settings {
        if set_active {
            settings.active_palette = palette
                .load_path()
                .map(ToOwned::to_owned);
        }
        
        let res = settings.write_to_load_path_if_new();
        if res.as_ref().map_err(already_exists).err().unwrap_or(false) {
            info!("Settings file already exists.");
            debug!("Settings {:?}", settings.load_path());
        } else {
            let _ = res.with_context(|| 
                if let Some(path) = settings.load_path() {
                    format!("Error writing settings file: {}", path.display())
                } else {
                    format!("Error writing settings file")
                })?;
        }
    }

    let res = palette.write_to_load_path_if_new();
    if res.as_ref().map_err(already_exists).err().unwrap_or(false) {
        info!("Palette file already exists.");
        debug!("Palette {:?}", palette.load_path());
    } else {
        let _ = res.with_context(|| 
            if let Some(path) = palette.load_path() {
                format!("Error writing palette file: {}", path.display())
            } else {
                format!("Error writing palette file")
            })?;
    }
    Ok(())
}



#[cfg(not(feature = "png"))]
fn write_png<'a>(palette: &Palette, selection: CellSelection<'a>, path: &Path)
    -> Result<(), anyhow::Error>
{
    Err(anyhow!("Export using PNG format is unsupported."))
}

#[cfg(feature = "png")]
fn write_png<'a>(palette: &Palette, selection: CellSelection<'a>, path: &Path)
    -> Result<(), anyhow::Error>
{
    let mut pal_data = Vec::new();
    let index_selection = selection.resolve(palette.inner());    
    for idx in index_selection {
        if let Ok(Some(c)) = palette.inner().color(&CellRef::Index(idx)) {
            pal_data.extend(&c.rgb_octets());
        }
    }

    let file = std::fs::File::create(path)?;
    let ref mut w = std::io::BufWriter::new(file);

    let mut encoder = png::Encoder::new(w, 1, 1);
    encoder.set_color(png::ColorType::Indexed);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.set_palette(pal_data);
    let mut writer = encoder.write_header()?;
    writer.write_image_data(&[0])?;
    println!("Palette exported to {}", path.display());
    Ok(())
}
