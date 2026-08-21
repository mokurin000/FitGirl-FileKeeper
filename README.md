# FitGirl-FileKeeper

Grab `filekeeper.net` direct links from `FitGirl-Repacks.site`, output as an aria2 input file.

FileKeeper is the experimental new file hoster, not widely supported as `fuckingfast.co`, but it's
much easier to scrape and need no cloudflare turnstile captcha.

## Inner implementation

`filekeeper.net` itself is behind cloudflare, and would `set-cookie` the `affiliate` once you get passed.

The `file_code` in `cookie` was only used for gathering file information on their website,
the actual `/download` POST specifies `file_code` in the form POST, to retrieve an 304 redirection
to the direct link.
