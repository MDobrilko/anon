use std::{
    collections::{HashMap, HashSet},
    path::Path,
};

use anyhow::Context;
use serde::{Deserialize, Serialize};
use tokio::sync::{RwLock, RwLockReadGuard};

use crate::bot::entities::Chat as TgChat;

pub struct Chats(RwLock<ChatsData>);

impl Chats {
    pub async fn open(file: &Path) -> anyhow::Result<Self> {
        let chats_array: Vec<ChatInfo> = match file.exists() {
            true => {
                let contents = tokio::fs::read(file).await?;

                serde_json::from_slice(&contents)?
            }
            false => vec![],
        };

        let mut users_to_chats: HashMap<i64, HashSet<i64>> = HashMap::new();
        let mut chats = HashMap::new();
        for chat in chats_array {
            for &user_id in &chat.members {
                users_to_chats.entry(user_id).or_default().insert(chat.id);
            }
            chats.insert(chat.id, chat);
        }

        Ok(Self(RwLock::new(ChatsData {
            users_to_chats,
            chats,
        })))
    }

    pub async fn get_chat(&self, chat_id: i64) -> Option<RwLockReadGuard<'_, ChatInfo>> {
        let guard = self.0.read().await;

        RwLockReadGuard::try_map(guard, |this| this.chats.get(&chat_id)).ok()
    }

    pub async fn get_user_chats(&self, user_id: i64) -> Vec<ChatInfo> {
        let all_chats = self.0.read().await;

        let Some(user_chats) = all_chats.users_to_chats.get(&user_id) else {
            return vec![];
        };

        user_chats
            .iter()
            .flat_map(|chat_id| all_chats.chats.get(chat_id).cloned())
            .collect()
    }

    pub async fn add_user_chat(&self, user_id: i64, chat: &TgChat) -> bool {
        let mut all_chats = self.0.write().await;

        let saved_chat = all_chats.chats.entry(chat.id).or_insert_with(|| ChatInfo {
            id: chat.id,
            title: chat.title.clone(),
            members: HashSet::new(),
        });

        if !saved_chat.members.insert(user_id) {
            return false;
        }

        all_chats
            .users_to_chats
            .entry(user_id)
            .or_default()
            .insert(chat.id)
    }

    pub async fn save(&self, file: &Path) -> anyhow::Result<()> {
        let chats_array: Vec<ChatInfo> = { self.0.read().await.chats.values().cloned().collect() };

        let serialized =
            serde_json::to_vec_pretty(&chats_array).context("Failed to serialize chats data")?;

        tokio::fs::write(file, serialized)
            .await
            .context("Failed to save chats data to file")?;

        Ok(())
    }
}

struct ChatsData {
    users_to_chats: HashMap<i64, HashSet<i64>>,
    chats: HashMap<i64, ChatInfo>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ChatInfo {
    pub id: i64,
    pub title: Option<String>,
    pub members: HashSet<i64>,
}
