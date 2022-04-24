mod configure;
mod create;
mod delete;
mod list;
pub use configure::configure;
pub use create::create;
pub use delete::delete;
pub use list::list;

fn pick_mod_loader(
    default: Option<&libium::config::structs::ModLoader>,
) -> std::io::Result<libium::config::structs::ModLoader> {
    let mut picker = dialoguer::Select::with_theme(&*crate::THEME);
    picker
        .with_prompt("Which mod loader do you use?")
        .items(&["Quilt", "Fabric", "Forge"]);
    if let Some(default) = default {
        picker.default(match default {
            libium::config::structs::ModLoader::Quilt => 0,
            libium::config::structs::ModLoader::Fabric => 1,
            libium::config::structs::ModLoader::Forge => 2,
        });
    }
    match picker.interact()? {
        0 => Ok(libium::config::structs::ModLoader::Quilt),
        1 => Ok(libium::config::structs::ModLoader::Fabric),
        2 => Ok(libium::config::structs::ModLoader::Forge),
        _ => unreachable!(),
    }
}
