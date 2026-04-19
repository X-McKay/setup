//! Component registry. Populated by `build()` at startup.

use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::sync::Arc;

use super::Component;

pub struct Registry {
    components: HashMap<String, Arc<dyn Component>>,
}

impl Registry {
    /// Build the full registry by wiring every component implementation.
    /// This is the ONE place that knows about every component in the system.
    pub fn build() -> Self {
        let mut r = Self {
            components: HashMap::new(),
        };
        r.register(Arc::new(super::apt::Apt));
        r.register(Arc::new(super::bottom::Bottom));
        r.register(Arc::new(super::docker::Docker));
        r.register(Arc::new(super::gh::Gh));
        r.register(Arc::new(super::glow::Glow));
        r.register(Arc::new(super::just::Just));
        r.register(Arc::new(super::lazygit::Lazygit));
        r.register(Arc::new(super::mise::Mise));
        r.register(Arc::new(super::tools::Tools));
        r
    }

    pub fn register(&mut self, c: Arc<dyn Component>) {
        let id = c.id().to_string();
        if self.components.insert(id.clone(), c).is_some() {
            panic!("duplicate component registration: {}", id);
        }
    }

    pub fn get(&self, id: &str) -> Result<Arc<dyn Component>> {
        self.components
            .get(id)
            .cloned()
            .ok_or_else(|| anyhow!("unknown component: {}", id))
    }

    pub fn ids(&self) -> Vec<String> {
        let mut v: Vec<_> = self.components.keys().cloned().collect();
        v.sort();
        v
    }

    /// Validate that every id in the manifest has a registered implementation,
    /// and every registered implementation has a manifest entry.
    pub fn validate_against(&self, manifest: &crate::manifest::schema::Manifest) -> Result<()> {
        use std::collections::HashSet;
        let reg: HashSet<_> = self.components.keys().cloned().collect();
        let man: HashSet<_> = manifest.components.iter().map(|c| c.id.clone()).collect();

        let missing_impls: Vec<_> = man.difference(&reg).cloned().collect();
        let orphan_impls: Vec<_> = reg.difference(&man).cloned().collect();

        if !missing_impls.is_empty() || !orphan_impls.is_empty() {
            let mut msg = String::new();
            if !missing_impls.is_empty() {
                msg.push_str(&format!(
                    "manifest components without Rust impl: {};\n",
                    missing_impls.join(", ")
                ));
            }
            if !orphan_impls.is_empty() {
                msg.push_str(&format!(
                    "Rust impls without manifest entry: {}",
                    orphan_impls.join(", ")
                ));
            }
            anyhow::bail!("registry/manifest mismatch:\n{}", msg);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    struct FakeA;
    impl Component for FakeA {
        fn id(&self) -> &str {
            "a"
        }
        fn is_installed(&self) -> Result<bool> {
            Ok(false)
        }
        fn install(&self) -> Result<()> {
            Ok(())
        }
    }

    fn assert_registered(id: &str) {
        let r = Registry::build();
        let c = r.get(id).unwrap();
        assert_eq!(c.id(), id);
    }

    #[test]
    fn register_and_get() {
        let mut r = Registry {
            components: HashMap::new(),
        };
        r.register(Arc::new(FakeA));
        assert!(r.get("a").is_ok());
        assert!(r.get("missing").is_err());
    }

    #[test]
    fn ids_is_sorted() {
        let mut r = Registry {
            components: HashMap::new(),
        };
        r.register(Arc::new(FakeA));
        assert_eq!(r.ids(), vec!["a"]);
    }

    #[test]
    fn apt_is_registered() {
        assert_registered("apt");
    }

    #[test]
    fn tools_is_registered() {
        assert_registered("tools");
    }

    #[test]
    fn mise_is_registered() {
        assert_registered("mise");
    }

    #[test]
    fn docker_is_registered() {
        assert_registered("docker");
    }

    #[test]
    fn lazygit_is_registered() {
        assert_registered("lazygit");
    }

    #[test]
    fn just_is_registered() {
        assert_registered("just");
    }

    #[test]
    fn glow_is_registered() {
        assert_registered("glow");
    }

    #[test]
    fn bottom_is_registered() {
        assert_registered("bottom");
    }

    #[test]
    fn gh_is_registered() {
        assert_registered("gh");
    }

    #[test]
    #[should_panic(expected = "duplicate")]
    fn duplicate_registration_panics() {
        let mut r = Registry {
            components: HashMap::new(),
        };
        r.register(Arc::new(FakeA));
        r.register(Arc::new(FakeA));
    }
}
