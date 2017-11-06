# dewey

`dewey` is a simple version parser and comperator that aims to be compatible
to [NetBSD](http://netbsd.org) and [xbps](https://github.com/voidlinux/xbps)'
comperator implementation.

`dewey` not only parses `.`-seperated versions but other common patterns such
as `X.XalphaX`, `X.XrcX`, and `X.X.Xpl1`

## example

```rust
let stable = "1.0".version();
let pre = "1.0pre1".version()
let pl = "1.0pl1".version();
assert!(stable > pre);
assert!(pl > stable);
assert!(pl > pre);
```

## supported seperators

* Revision: example: `1.0_1`
* Alpha: `0.0alpha1`
* Beta: `0.0beta1`
* Pre: `0.0pre1`
* Rc: `0.0rc1`
* PatchLevel: `0.0pl1`
* Dot: `1.0`
