/*
   Copyright 2024 Sean Cribbs

   Licensed under the Apache License, Version 2.0 (the "License");
   you may not use this file except in compliance with the License.
   You may obtain a copy of the License at

       http://www.apache.org/licenses/LICENSE-2.0

   Unless required by applicable law or agreed to in writing, software
   distributed under the License is distributed on an "AS IS" BASIS,
   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
   See the License for the specific language governing permissions and
   limitations under the License.
*/
use crate::language_server::*;
use zed_extension_api as zed;

mod language_server;

struct DhallExtension {
    language_server: Option<DhallLanguageServer>,
}

impl zed::Extension for DhallExtension {
    fn new() -> Self
    where
        Self: Sized,
    {
        Self {
            language_server: None,
        }
    }

    fn language_server_command(
        &mut self,
        language_server_id: &zed::LanguageServerId,
        worktree: &zed::Worktree,
    ) -> zed::Result<zed::Command> {
        if language_server_id.as_ref() == DhallLanguageServer::LANGUAGE_SERVER_ID {
            let language_server = self
                .language_server
                .get_or_insert_with(DhallLanguageServer::new);
            language_server.language_server_command(language_server_id, worktree)
        } else {
            Err(format!("unknown language server: {language_server_id}"))
        }
    }
}

zed::register_extension!(DhallExtension);
