use std::sync::Arc;

use snafu::{ResultExt, Whatever};

pub use add_medium_item::{AddMediumItemInput, AddMediumItemType};
pub use create_medium::CreateMediumInput;
pub use find_medium::{FindAllMediaInput, GetMediumFileType};
pub use user::CreateUserInput;

mod add_medium_item;
mod album;
mod create_medium;
mod find_medium;
mod path;
mod user;
