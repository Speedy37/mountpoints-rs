# mountpoints - List mount points (windows, linux, macos)

## Example

```rust
use mountpoints::mount_points;

fn main() {
    for mount_point in mount_points().unwrap() {
        println!("{}", mount_point.display());
    }
}
```

**Windows output:**

```
C:\
C:\MyLittleMountPoint
D:\
```

**Linux output:**

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

**Macos output:**

```
/
/dev
/System/Volumes/Data
/private/var/vm
/System/Volumes/Data/home
/Volumes/VMware Shared Folders
```
