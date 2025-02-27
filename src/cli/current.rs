use color_eyre::eyre::Result;

use crate::cli::command::Command;

use crate::config::Config;
use crate::output::Output;
use crate::tool::Tool;
use crate::toolset::{Toolset, ToolsetBuilder};

/// Shows current active and installed runtime versions
///
/// This is similar to `rtx ls --current`, but this only shows the runtime
/// and/or version. It's designed to fit into scripts more easily.
///
/// This command is currently hidden as we may be able to merge it with `ls`
/// and it's confusing to have two commands that do similar things.
/// It will probably never be removed in order to retain some compatibility with asdf.
#[derive(Debug, clap::Args)]
#[clap(verbatim_doc_comment, hide = true, after_long_help = AFTER_LONG_HELP)]
pub struct Current {
    /// Plugin to show versions of
    /// e.g.: ruby, nodejs
    #[clap()]
    plugin: Option<String>,
}

impl Command for Current {
    fn run(self, mut config: Config, out: &mut Output) -> Result<()> {
        let ts = ToolsetBuilder::new().build(&mut config)?;
        match &self.plugin {
            Some(plugin_name) => match config.tools.get(plugin_name) {
                Some(plugin) => self.one(&config, ts, out, plugin),
                None => {
                    warn!("Plugin {} is not installed", plugin_name);
                    Ok(())
                }
            },
            None => self.all(&config, ts, out),
        }
    }
}

impl Current {
    fn one(&self, config: &Config, ts: Toolset, out: &mut Output, tool: &Tool) -> Result<()> {
        if !tool.is_installed() {
            warn!("Plugin {} is not installed", tool.name);
            return Ok(());
        }
        match ts
            .list_versions_by_plugin(config)
            .into_iter()
            .find(|(p, _)| p.name == tool.name)
        {
            Some((_, versions)) => {
                rtxprintln!(
                    out,
                    "{}",
                    versions
                        .iter()
                        .map(|v| v.version.to_string())
                        .collect::<Vec<_>>()
                        .join(" ")
                );
            }
            None => {
                warn!("Plugin {} does not have a version set", tool.name);
            }
        };
        Ok(())
    }

    fn all(&self, config: &Config, ts: Toolset, out: &mut Output) -> Result<()> {
        for (plugin, versions) in ts.list_versions_by_plugin(config) {
            if versions.is_empty() {
                continue;
            }
            for tv in versions {
                if !plugin.is_version_installed(tv) {
                    let source = ts.versions.get(&tv.plugin_name).unwrap().source.clone();
                    warn!(
                        "{}@{} is specified in {}, but not installed",
                        tv.plugin_name, &tv.version, &source
                    );
                }
            }
            rtxprintln!(
                out,
                "{} {}",
                &plugin.name,
                versions
                    .iter()
                    .map(|v| v.version.to_string())
                    .collect::<Vec<_>>()
                    .join(" ")
            );
        }
        Ok(())
    }
}

static AFTER_LONG_HELP: &str = color_print::cstr!(
    r#"<bold><underline>Examples:</underline></bold>
  # outputs `.tool-versions` compatible format
  $ <bold>rtx current</bold>
  python 3.11.0 3.10.0
  shfmt 3.6.0
  shellcheck 0.9.0
  nodejs 18.13.0

  $ <bold>rtx current nodejs</bold>
  18.13.0

  # can output multiple versions
  $ <bold>rtx current python</bold>
  3.11.0 3.10.0
"#
);

#[cfg(test)]
mod tests {
    use std::env;

    use crate::{assert_cli, assert_cli_snapshot};

    #[test]
    fn test_current() {
        assert_cli_snapshot!("current");
    }

    #[test]
    fn test_current_with_runtimes() {
        assert_cli_snapshot!("current", "tiny");
    }

    #[test]
    fn test_current_missing() {
        assert_cli!("uninstall", "dummy@1.0.1");

        env::set_var("RTX_DUMMY_VERSION", "1.1.0");
        assert_cli_snapshot!("current");

        env::remove_var("RTX_DUMMY_VERSION");
    }
}
