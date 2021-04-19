# mountpoints - List mount points (windows, linux, ...)

## Example

```rust
use mountpoints::mount_points;

fn main() {
    for mount_point in mount_points().unwrap() {
        println!("{}", mount_point.display());
    }
}
```

__Windows output:__

```
C:\
C:\MyLittleMountPoint
D:\
```

__Linux output:__

```
/mnt/wsl
/init
/dev
/dev/pts
/run
/run/lock
/run/shm
/run/user
/proc/sys/fs/binfmt_misc
/sys/fs/cgroup
/sys/fs/cgroup/unified
/mnt/c
/mnt/d
```
