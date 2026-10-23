use gemini_rust::InteractionTool;
use serde_json::{Value, json};
use serenity::all::Permissions;
use strum::{Display, EnumString, VariantArray};

use disaity_core::{Context, Error};

/// read only tools that turn names like "bob" into ids for the ai
#[derive(Display, Debug, Clone, Copy, Eq, Hash, PartialEq, EnumString, VariantArray)]
#[strum(serialize_all = "snake_case")]
pub(super) enum LookUp {
    FindMember,
    ListChannels,
    ListRoles,
}

impl LookUp {
    const LIST_LIMIT: usize = 50;

    /// every lookup as a gemini tool
    pub fn declarations() -> Vec<InteractionTool> {
        Self::VARIANTS.iter().map(Self::declaration).collect()
    }

    fn declaration(&self) -> InteractionTool {
        let (about, parameters) = match self {
            Self::FindMember => (
                "Find members by username or nickname. Returns up to 10 with their ids.",
                Self::query("name or nickname", true),
            ),

            Self::ListChannels => (
                "List channels with id, name and kind.",
                Self::query("optional part of a channel name", false),
            ),

            Self::ListRoles => (
                "List roles with id, name and position.",
                Self::query("optional part of a role name", false),
            ),
        };

        InteractionTool::function(self.to_string(), about, parameters)
    }

    /// the one `query` argument every lookup takes
    fn query(about: &str, required: bool) -> Value {
        let required: &[&str] = if required { &["query"] } else { &[] };

        json!({
            "type": "object",
            "properties": { "query": { "type": "string", "description": about } },
            "required": required,
        })
    }

    /// read only, never changes anything
    pub async fn run(self, ctx: Context<'_>, args: &Value) -> Result<Value, Error> {
        let author = ctx
            .author_member()
            .await
            .ok_or("This command only works in servers.")?
            .into_owned();

        let guild = ctx
            .guild()
            .ok_or("This command only works in servers.")?
            .clone();

        let query = args["query"].as_str().unwrap_or_default().to_lowercase();

        let hit = |name: &str| name.to_lowercase().contains(&query);

        Ok(match self {
            Self::FindMember => {
                let found = guild
                    .id
                    .search_members(ctx.http(), &query, Some(10))
                    .await?;

                json!({
                    "members": found.iter()
                        .map(|m| json!({ "id": m.user.id.to_string(), "username": m.user.name, "display_name": m.display_name() }))
                        .collect::<Vec<_>>(),
                })
            }

            Self::ListChannels => json!({
                "channels": guild.channels.values()
                    // don't reveal channels the requester can't see
                    .filter(|c| guild.user_permissions_in(c, &author).contains(Permissions::VIEW_CHANNEL))
                    .filter(|c| hit(&c.name))
                    .take(Self::LIST_LIMIT)
                    .map(|c| json!({ "id": c.id.to_string(), "name": c.name, "kind": format!("{:?}", c.kind) }))
                    .collect::<Vec<_>>(),
            }),

            Self::ListRoles => json!({
                "roles": guild.roles.values()
                    .filter(|r| hit(&r.name))
                    .take(Self::LIST_LIMIT)
                    .map(|r| json!({ "id": r.id.to_string(), "name": r.name, "position": r.position }))
                    .collect::<Vec<_>>(),
            }),
        })
    }
}
