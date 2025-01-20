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

use std::fs;

use zed_extension_api::{self as zed, LanguageServerId, Result};

pub struct DhallLanguageServer {
    cached_binary_path: Option<String>,
}

impl DhallLanguageServer {
    pub const LANGUAGE_SERVER_ID: &'static str = "dhall";

    pub fn new() -> Self {
        Self {
            cached_binary_path: None,
        }
    }

    pub fn language_server_command(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        Ok(zed::Command {
            command: self.language_server_binary_path(language_server_id, worktree)?,
            args: vec![],
            env: Default::default(),
        })
    }

    fn language_server_binary_path(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<String> {
        let (platform, arch) = zed::current_platform();
        let binary_name = if let zed_extension_api::Os::Windows = platform {
            "dhall-lsp-server.exe"
        } else {
            "dhall-lsp-server"
        };

        if let Some(path) = worktree.which(binary_name) {
            return Ok(path);
        }

        if let Some(path) = &self.cached_binary_path {
            if fs::metadata(path).is_ok_and(|stat| stat.is_file()) {
                return Ok(path.clone());
            }
        }

        zed::set_language_server_installation_status(
            language_server_id,
            &zed::LanguageServerInstallationStatus::CheckingForUpdate,
        );
        let release = zed::latest_github_release(
            "dhall-lang/dhall-haskell",
            zed::GithubReleaseOptions {
                require_assets: true,
                pre_release: false,
            },
        )?;

        let (file_suffix, download_type) = match (platform, arch) {
            (zed::Os::Mac, zed::Architecture::Aarch64) => (
                "aarch64-darwin.tar.bz2",
                zed::DownloadedFileType::Uncompressed,
            ),
            (zed::Os::Mac, zed::Architecture::X8664) => (
                "x86_64-darwin.tar.bz2",
                zed::DownloadedFileType::Uncompressed,
            ),
            (zed::Os::Linux, zed::Architecture::X8664) => (
                "x86_64-linux.tar.bz2",
                zed::DownloadedFileType::Uncompressed,
            ),
            (zed::Os::Windows, zed::Architecture::X8664) => {
                ("x86_64-windows.zip", zed::DownloadedFileType::Zip)
            }
            (platform, arch) => {
                return Err(format!(
                    "unsupported platform/arch combination: {platform:?}/{arch:?}"
                ))
            }
        };
        let asset = release
            .assets
            .iter()
            .find(|asset| {
                asset.name.starts_with("dhall-lsp-server") && asset.name.ends_with(file_suffix)
            })
            .ok_or_else(|| format!("no asset found matching dhall-lsp-server-*-{file_suffix}"))?;
        let version_dir = format!("dhall-haskell-{}", release.version);

        let binary_path = format!("{version_dir}/bin/{binary_name}");
        let download_path = format!("{version_dir}/{}", asset.name);
        if !fs::metadata(&binary_path).is_ok_and(|stat| stat.is_file()) {
            zed::set_language_server_installation_status(
                language_server_id,
                &zed::LanguageServerInstallationStatus::Downloading,
            );

            zed::download_file(&asset.download_url, &version_dir, download_type)
                .map_err(|e| format!("failed to download file: {e}"))?;

            if download_type == zed::DownloadedFileType::Uncompressed {
                // These are .tar.bz2, we need to manually uncompress them
                let exit_status = std::process::Command::new("tar")
                    .arg("-xf")
                    .arg(&download_path)
                    .status()
                    .map_err(|e| format!("failed to decompress {download_path}: {e:?}"))?;
                if !exit_status.success() {
                    return Err(format!(
                        "failed to decompress {download_path}: status {exit_status:?}"
                    ));
                }
            }

            let entries =
                fs::read_dir(".").map_err(|e| format!("failed to list working directory {e}"))?;
            for entry in entries {
                let entry = entry.map_err(|e| format!("failed to load directory entry {e}"))?;
                if entry.file_name().to_str() != Some(&version_dir) {
                    fs::remove_dir_all(entry.path()).ok();
                }
            }
        }

        self.cached_binary_path = Some(binary_path.clone());
        Ok(binary_path)
    }
}
