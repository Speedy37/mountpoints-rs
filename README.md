# mountpoints - List mount points (windows, linux, macos)

## Example

```rust
use mountpoints::mountpaths;

fn main() {
    for mountpath in mountpaths().unwrap() {
        println!("{}", mountpath.display());
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
