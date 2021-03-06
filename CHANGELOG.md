# Changelog

## 0.1.0

initial release

### 0.1.1

fix docs

## 0.2.0

this version was mostly focussed on improving `alpha_beta::Bot::select` and removing code duplication by changing `tree::Node`.

- `Game`
  - `type Player` must now be `Copy`
  - `execute`, `look_ahead` and `actions` now take `player` by value
  - add provided methods `is_lower_bound` and `is_upper_bound`

- `tree::Node`
  - all methods are now non const.
  - `children` is renamed to `with_children` and does not require the given reference to be `'static`
  - add methods `push_child`, `child_count`, `from_bytes` and `is_leaf`

## 0.3.0

it is now possible to get the expected fitness and path from `Bot` using `detailed_select`.

- `Game`
  - `type Action` must now be `Clone`.
- `Bot`
  - added `fn detailed_select`.
