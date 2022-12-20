# sitemap-generator-rs
Simple sitemap generator written in Rust.

```
$ ./target/debug/sitemap-gen --help
Usage: sitemap-gen [OPTIONS] --site <URI>

Options:
  -t, --site <URI>         [env: SITE_URI=]
  -x, --exclude <EXCLUDE>
  -h, --help               Print help information

$ sitemap-gen -t https://ntalbs.github.io -x /tags -x archives -x page
```

If the environment variable `SITE_URI` is set, it will use it unless you provide a site URI.

```
$ export SITE_URI=https://ntalbs.github.io
$ sitemap-gen -x /tags -x archives -x page
```
