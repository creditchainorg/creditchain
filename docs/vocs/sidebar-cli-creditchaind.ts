import type { SidebarItem } from "./types";

export const creditchaindCliSidebar: SidebarItem = {
    text: "creditchaind",
    link: "/cli/creditchaind",
    collapsed: false,
    items: [
        {
            text: "creditchaind node",
            link: "/cli/creditchaind/node"
        },
        {
            text: "creditchaind init",
            link: "/cli/creditchaind/init"
        },
        {
            text: "creditchaind init-state",
            link: "/cli/creditchaind/init-state"
        },
        {
            text: "creditchaind import",
            link: "/cli/creditchaind/import"
        },
        {
            text: "creditchaind import-era",
            link: "/cli/creditchaind/import-era"
        },
        {
            text: "creditchaind export-era",
            link: "/cli/creditchaind/export-era"
        },
        {
            text: "creditchaind dump-genesis",
            link: "/cli/creditchaind/dump-genesis"
        },
        {
            text: "creditchaind db",
            link: "/cli/creditchaind/db",
            collapsed: true,
            items: [
                {
                    text: "creditchaind db stats",
                    link: "/cli/creditchaind/db/stats"
                },
                {
                    text: "creditchaind db list",
                    link: "/cli/creditchaind/db/list"
                },
                {
                    text: "creditchaind db checksum",
                    link: "/cli/creditchaind/db/checksum",
                    collapsed: true,
                    items: [
                        {
                            text: "creditchaind db checksum mdbx",
                            link: "/cli/creditchaind/db/checksum/mdbx"
                        },
                        {
                            text: "creditchaind db checksum static-file",
                            link: "/cli/creditchaind/db/checksum/static-file"
                        },
                        {
                            text: "creditchaind db checksum rocksdb",
                            link: "/cli/creditchaind/db/checksum/rocksdb"
                        }
                    ]
                },
                {
                    text: "creditchaind db copy",
                    link: "/cli/creditchaind/db/copy"
                },
                {
                    text: "creditchaind db diff",
                    link: "/cli/creditchaind/db/diff"
                },
                {
                    text: "creditchaind db get",
                    link: "/cli/creditchaind/db/get",
                    collapsed: true,
                    items: [
                        {
                            text: "creditchaind db get mdbx",
                            link: "/cli/creditchaind/db/get/mdbx"
                        },
                        {
                            text: "creditchaind db get static-file",
                            link: "/cli/creditchaind/db/get/static-file"
                        },
                        {
                            text: "creditchaind db get rocksdb",
                            link: "/cli/creditchaind/db/get/rocksdb"
                        }
                    ]
                },
                {
                    text: "creditchaind db drop",
                    link: "/cli/creditchaind/db/drop"
                },
                {
                    text: "creditchaind db clear",
                    link: "/cli/creditchaind/db/clear",
                    collapsed: true,
                    items: [
                        {
                            text: "creditchaind db clear mdbx",
                            link: "/cli/creditchaind/db/clear/mdbx"
                        },
                        {
                            text: "creditchaind db clear static-file",
                            link: "/cli/creditchaind/db/clear/static-file"
                        }
                    ]
                },
                {
                    text: "creditchaind db repair-trie",
                    link: "/cli/creditchaind/db/repair-trie"
                },
                {
                    text: "creditchaind db static-file-header",
                    link: "/cli/creditchaind/db/static-file-header",
                    collapsed: true,
                    items: [
                        {
                            text: "creditchaind db static-file-header block",
                            link: "/cli/creditchaind/db/static-file-header/block"
                        },
                        {
                            text: "creditchaind db static-file-header path",
                            link: "/cli/creditchaind/db/static-file-header/path"
                        }
                    ]
                },
                {
                    text: "creditchaind db version",
                    link: "/cli/creditchaind/db/version"
                },
                {
                    text: "creditchaind db path",
                    link: "/cli/creditchaind/db/path"
                },
                {
                    text: "creditchaind db settings",
                    link: "/cli/creditchaind/db/settings",
                    collapsed: true,
                    items: [
                        {
                            text: "creditchaind db settings get",
                            link: "/cli/creditchaind/db/settings/get"
                        },
                        {
                            text: "creditchaind db settings set",
                            link: "/cli/creditchaind/db/settings/set",
                            collapsed: true,
                            items: [
                                {
                                    text: "creditchaind db settings set v2",
                                    link: "/cli/creditchaind/db/settings/set/v2"
                                }
                            ]
                        }
                    ]
                },
                {
                    text: "creditchaind db prune-checkpoints",
                    link: "/cli/creditchaind/db/prune-checkpoints",
                    collapsed: true,
                    items: [
                        {
                            text: "creditchaind db prune-checkpoints get",
                            link: "/cli/creditchaind/db/prune-checkpoints/get"
                        },
                        {
                            text: "creditchaind db prune-checkpoints set",
                            link: "/cli/creditchaind/db/prune-checkpoints/set"
                        }
                    ]
                },
                {
                    text: "creditchaind db stage-checkpoints",
                    link: "/cli/creditchaind/db/stage-checkpoints",
                    collapsed: true,
                    items: [
                        {
                            text: "creditchaind db stage-checkpoints get",
                            link: "/cli/creditchaind/db/stage-checkpoints/get"
                        },
                        {
                            text: "creditchaind db stage-checkpoints set",
                            link: "/cli/creditchaind/db/stage-checkpoints/set"
                        }
                    ]
                },
                {
                    text: "creditchaind db account-storage",
                    link: "/cli/creditchaind/db/account-storage"
                },
                {
                    text: "creditchaind db state",
                    link: "/cli/creditchaind/db/state"
                },
                {
                    text: "creditchaind db migrate-v2",
                    link: "/cli/creditchaind/db/migrate-v2"
                }
            ]
        },
        {
            text: "creditchaind download",
            link: "/cli/creditchaind/download"
        },
        {
            text: "creditchaind snapshot-manifest",
            link: "/cli/creditchaind/snapshot-manifest"
        },
        {
            text: "creditchaind stage",
            link: "/cli/creditchaind/stage",
            collapsed: true,
            items: [
                {
                    text: "creditchaind stage run",
                    link: "/cli/creditchaind/stage/run"
                },
                {
                    text: "creditchaind stage drop",
                    link: "/cli/creditchaind/stage/drop"
                },
                {
                    text: "creditchaind stage dump",
                    link: "/cli/creditchaind/stage/dump",
                    collapsed: true,
                    items: [
                        {
                            text: "creditchaind stage dump execution",
                            link: "/cli/creditchaind/stage/dump/execution"
                        },
                        {
                            text: "creditchaind stage dump storage-hashing",
                            link: "/cli/creditchaind/stage/dump/storage-hashing"
                        },
                        {
                            text: "creditchaind stage dump account-hashing",
                            link: "/cli/creditchaind/stage/dump/account-hashing"
                        },
                        {
                            text: "creditchaind stage dump merkle",
                            link: "/cli/creditchaind/stage/dump/merkle"
                        }
                    ]
                },
                {
                    text: "creditchaind stage unwind",
                    link: "/cli/creditchaind/stage/unwind",
                    collapsed: true,
                    items: [
                        {
                            text: "creditchaind stage unwind to-block",
                            link: "/cli/creditchaind/stage/unwind/to-block"
                        },
                        {
                            text: "creditchaind stage unwind num-blocks",
                            link: "/cli/creditchaind/stage/unwind/num-blocks"
                        }
                    ]
                }
            ]
        },
        {
            text: "creditchaind p2p",
            link: "/cli/creditchaind/p2p",
            collapsed: true,
            items: [
                {
                    text: "creditchaind p2p header",
                    link: "/cli/creditchaind/p2p/header"
                },
                {
                    text: "creditchaind p2p body",
                    link: "/cli/creditchaind/p2p/body"
                },
                {
                    text: "creditchaind p2p rlpx",
                    link: "/cli/creditchaind/p2p/rlpx",
                    collapsed: true,
                    items: [
                        {
                            text: "creditchaind p2p rlpx ping",
                            link: "/cli/creditchaind/p2p/rlpx/ping"
                        }
                    ]
                },
                {
                    text: "creditchaind p2p bootnode",
                    link: "/cli/creditchaind/p2p/bootnode"
                },
                {
                    text: "creditchaind p2p enode",
                    link: "/cli/creditchaind/p2p/enode"
                }
            ]
        },
        {
            text: "creditchaind config",
            link: "/cli/creditchaind/config"
        },
        {
            text: "creditchaind prune",
            link: "/cli/creditchaind/prune"
        },
        {
            text: "creditchaind re-execute",
            link: "/cli/creditchaind/re-execute"
        }
    ]
};
