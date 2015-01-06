
# scope-time

Time function calls and save them to a global DB

## Example

```rust
extern crate scope-time;
use scope-time::{TimeIt,TimeFileSave};
{
  TimeIt::new("f1");
 // do stuff
}
TimeFileSave::new("profile.txt");
```
