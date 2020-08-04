+++
title = "Localization"
weight = 100
+++

Zola uses the [Fluent](https://www.projectfluent.org) localization
framework to simplify writing multilingual templates and themes. It is
used in Mozilla products and has many advanced features, not to mention
a native Rust implementation.

This documentation will only talk about the basic features of Fluent and
how it is used in Zola. For a more in-depth list of what you can do with
it, read the official [syntax guide](https://www.projectfluent.org/fluent/guide).

## The `fluent` function
The `fluent` function returns a localized message. Its arguments are:
- `key`: name of the message. It takes a string value. (**required**)
- `lang`: the language to get the key for. It takes a string value that
  is a valid language identifier. If not specified, the site's
  `default_language` will be used. (optional) <!-- FIXME: page's
  language-->
- Any other arguments are passed as
  [variables](https://www.projectfluent.org/fluent/guide/variables.html)
  to the message.

### Examples

#### Simple message
Given the Fluent files
```fluent
# locales/en/example.ftl
next-page = Next page

# locales/fr/example.ftl
next-page = ?????
```
this snippet
```jinja2
{{/* fluent(key="next-page") */}}
{{/* fluent(key="next-page", lang="fr") */}}
```
will produce:
```
Next page
?????
```

#### Message with arguments
Given the Fluent file
```fluent
# locales/en/example.ftl
reading-time-seconds =
    { $seconds ->
        [one] Reading time: { $seconds } seconds
        *[other] Reading time: one second
    }
```
this template
```jinja2
{{/* fluent(key="reading-time-seconds", seconds=1) */}}
{{/* fluent(key="reading-time-seconds", seconds=20) */}}
```
will produce:
```
Reading time: one second
Reading time: 20 seconds
```

#### Message with markup
Given the Fluent files
```fluent
# locales/common.ftl
-program = Zola
    .website = https://getzola.org

# locales/en/example.ftl
-link = <a href="{ -program.website }">{ -program }</a>
zola-credit = Made using { -link }, the best static site generator.
```
this template
```jinja2
{{ fluent(key="zola-credit") | safe }}
```
will produce the following HTML output:
```html
Made using <a href="https://getzola.org">Zola</a>, the best static site
generator.
```

### Language negotiation
If the specified key was not found among the given language's messages,
Zola first tries getting it from a variant of that language. Variants
have identifiers that only differ in the *region* (i.e. `en-US` and
`en-GB`, *script* (i.e. `sr-Cyrl` and `sr-Latn`) or *variant* subtags.
Then, `common.ftl` is searched. If none of these turn up with a result,
Zola will attempt to use the site's default language. If it still can't
find the message, an error will be displayed.

The exact algorithm can be found in the underlying library's
[API documentation](https://docs.rs/fluent-langneg/0.13.0/fluent_langneg/negotiate/index.html).

## Use in themes
<!-- FIXME -->
Currently, themes can't add their own localizations. This is expected
change in the future, as it's a basic feature. Users will be able to
override these translations, so some kind of prefixing is preferred.
