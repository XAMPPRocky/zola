use std::collections::HashMap;

use tera::{Context, Tera};
use unic_langid::LanguageIdentifier;

use errors::{bail, Result};
use std::path::PathBuf;

static DEFAULT_TPL: &str = include_str!("default_tpl.html");

macro_rules! render_default_tpl {
    ($filename: expr, $url: expr) => {{
        let mut context = Context::new();
        context.insert("filename", $filename);
        context.insert("url", $url);
        Tera::one_off(DEFAULT_TPL, &context, true).map_err(std::convert::Into::into)
    }};
}

/// Renders the given template with the given context, but also ensures that, if the default file
/// is not found, it will look up for the equivalent template for the current theme if there is one.
/// Lastly, if it's a default template (index, section or page), it will just return an empty string
/// to avoid an error if there isn't a template with that name
pub fn render_template(
    name: &str,
    tera: &Tera,
    context: Context,
    theme: &Option<String>,
    base_path: &PathBuf,
) -> Result<String> {
    let mut template_name = "";
    let mut tera = tera.clone();
    let theme_template_name: String;

    // check if it is in the templates
    if tera.templates.contains_key(name) {
        template_name = name;
    }

    // check if it is part of a theme
    if let Some(ref t) = *theme {
        theme_template_name = format!("{}/templates/{}", t, name);
        if tera.templates.contains_key(&theme_template_name) {
            template_name = &theme_template_name;
        }
    }

    // check if it is part of ZOLA_TERA defaults
    let default_name = format!("__zola_builtins/{}", name);
    if tera.templates.contains_key(&default_name) {
        template_name = &default_name;
    }

    if !template_name.is_empty() {
        // WIP: `fluent` function that's aware of the page's language and allows inherintance
        // of `.ftl` files
        if let Some(lang) = context.get("lang") {
            let lang = match lang.as_str() {
                // FIXME: error handling. Should not fail because it's already been deserailized once
                Some(id) => id.parse::<LanguageIdentifier>().unwrap(),
                None => bail!("Can't get page language"),
            };

            // base_path is not set for internal templates like shortcodes
            if !base_path.to_string_lossy().is_empty() {
                // fluent-templates does not yet support choosing which files to load
                /*
                let _template_chain = {
                    let mut tmp: Vec<&str> = Vec::new();
                    tmp.push(template_name);
                    let parents: Vec<&str> =
                    tera.templates.get(template_name).unwrap().parents.iter().map(|s| &**s).collect();
                    tmp.extend(parents);
                    tmp
                };
                */

                // TODO: map templates to .ftl files
                // TODO: load locales from themes too. fluent-templates only supports 1 search path

                let site_locales = {
                    let mut tmp = base_path.clone();
                    tmp.push("/locales");
                    tmp
                };

                if std::fs::metadata(&site_locales).is_ok() {
                    let loader = fluent_templates::ArcLoader::builder(&site_locales, lang)
                        // TODO: put locales/common.ftl if exists
                        .shared_resources(None)
                        .build()
                        .map_err(|e| e.to_string())?;

                    tera.register_function("fluent", fluent_templates::FluentLoader::new(loader));
                }
            }
        }

        return tera.render(template_name, &context).map_err(std::convert::Into::into);
    }

    // maybe it's a default one?
    match name {
        "index.html" | "section.html" => render_default_tpl!(
            name,
            "https://www.getzola.org/documentation/templates/pages-sections/#section-variables"
        ),
        "page.html" => render_default_tpl!(
            name,
            "https://www.getzola.org/documentation/templates/pages-sections/#page-variables"
        ),
        "single.html" | "list.html" => {
            render_default_tpl!(name, "https://www.getzola.org/documentation/templates/taxonomies/")
        }
        _ => bail!("Tried to render `{}` but the template wasn't found", name),
    }
}

/// Rewrites the path of duplicate templates to include the complete theme path
/// Theme templates  will be injected into site templates, with higher priority for site
/// templates. To keep a copy of the template in case it's being extended from a site template
/// of the same name, we reinsert it with the theme path prepended
pub fn rewrite_theme_paths(tera_theme: &mut Tera, theme: &str) {
    let theme_basepath = format!("{}/templates/", theme);
    let mut new_templates = HashMap::new();
    for (key, template) in &tera_theme.templates {
        let mut tpl = template.clone();
        tpl.name = format!("{}{}", theme_basepath, key);
        new_templates.insert(tpl.name.clone(), tpl);
    }
    // Contrary to tera.extend, hashmap.extend does replace existing keys
    // We can safely extend because there's no conflicting paths anymore
    tera_theme.templates.extend(new_templates);
}

#[cfg(test)]
mod tests {
    use super::rewrite_theme_paths;
    use tera::Tera;

    #[test]
    fn can_rewrite_all_paths_of_theme() {
        let mut tera = Tera::parse("test-templates/*.html").unwrap();
        rewrite_theme_paths(&mut tera, "hyde");
        // special case to make the test work: we also rename the files to
        // match the imports
        for (key, val) in &tera.templates.clone() {
            tera.templates.insert(format!("hyde/templates/{}", key), val.clone());
        }
        // Adding our fake base
        tera.add_raw_template("base.html", "Hello").unwrap();
        tera.build_inheritance_chains().unwrap();

        assert_eq!(
            tera.templates["hyde/templates/index.html"].parent,
            Some("base.html".to_string())
        );
        assert_eq!(
            tera.templates["hyde/templates/child.html"].parent,
            Some("index.html".to_string())
        );
    }
}
