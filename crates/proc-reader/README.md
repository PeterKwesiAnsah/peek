# peek-proc-reader

Linux `/proc` filesystem reader used by the [Peek](https://github.com/ankittk/peek) process intelligence tool.

Provides typed reading of `/proc/<pid>/*` (stat, status, cmdline, environ, fd, cgroup, limits, syscall, etc.) and error types for consumers. Used by `peek-core` on Linux.

Part of the [peek](https://github.com/ankittk/peek) workspace.
