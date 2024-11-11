/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs Ltd <hello@stalw.art>
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-SEL
 */

use super::*;

const PGSQL_NAME: &str =
    "SELECT name, type, secret, description, quota FROM accounts WHERE name = $1 AND active = true";
const PGSQL_MEMBERS: &str = "SELECT member_of FROM group_members WHERE name = $1";
const PGSQL_RECIPIENTS: &str = "SELECT name FROM emails WHERE address = $1 ORDER BY name ASC";
const PGSQL_EMAILS: &str =
    "SELECT address FROM emails WHERE name = $1 AND type != 'list' ORDER BY type DESC, address ASC";
const PGSQL_VRFY : &str = "SELECT address FROM emails WHERE address LIKE '%' || $1 || '%' AND type = 'primary' ORDER BY address LIMIT 5";
const PGSQL_EXPN : &str = "SELECT p.address FROM emails AS p JOIN emails AS l ON p.name = l.name WHERE p.type = 'primary' AND l.address = $1 AND l.type = 'list' ORDER BY p.address LIMIT 50";
const PGSQL_DOMAINS: &str = "SELECT 1 FROM emails WHERE address LIKE '%@' || $1 LIMIT 1";

const MYSQL_NAME: &str =
    "SELECT name, type, secret, description, quota FROM accounts WHERE name = ? AND active = true";
const MYSQL_MEMBERS: &str = "SELECT member_of FROM group_members WHERE name = ?";
const MYSQL_RECIPIENTS: &str = "SELECT name FROM emails WHERE address = ? ORDER BY name ASC";
const MYSQL_EMAILS: &str =
    "SELECT address FROM emails WHERE name = ? AND type != 'list' ORDER BY type DESC, address ASC";
const MYSQL_VRFY : &str = "SELECT address FROM emails WHERE address LIKE CONCAT('%', ?, '%') AND type = 'primary' ORDER BY address LIMIT 5";
const MYSQL_EXPN : &str = "SELECT p.address FROM emails AS p JOIN emails AS l ON p.name = l.name WHERE p.type = 'primary' AND l.address = ? AND l.type = 'list' ORDER BY p.address LIMIT 50";
const MYSQL_DOMAINS: &str = "SELECT 1 FROM emails WHERE address LIKE CONCAT('%@', ?) LIMIT 1";

const SQLITE_NAME: &str =
    "SELECT name, type, secret, description, quota FROM accounts WHERE name = ? AND active = true";
const SQLITE_MEMBERS: &str = "SELECT member_of FROM group_members WHERE name = ?";
const SQLITE_RECIPIENTS: &str = "SELECT name FROM emails WHERE address = ?";
const SQLITE_EMAILS: &str =
    "SELECT address FROM emails WHERE name = ? AND type != 'list' ORDER BY type DESC, address ASC";
const SQLITE_VRFY : &str = "SELECT address FROM emails WHERE address LIKE '%' || ? || '%' AND type = 'primary' ORDER BY address LIMIT 5";
const SQLITE_EXPN : &str = "SELECT p.address FROM emails AS p JOIN emails AS l ON p.name = l.name WHERE p.type = 'primary' AND l.address = ? AND l.type = 'list' ORDER BY p.address LIMIT 50";
const SQLITE_DOMAINS: &str = "SELECT 1 FROM emails WHERE address LIKE '%@' || ? LIMIT 1";

impl Builder<Schemas, ()> {
    pub fn build_store(self) -> Self {
        self.new_schema("store")
            .names("store", "stores")
            .prefix("store")
            .suffix("type")
            // Id
            .new_id_field()
            .label("Store Id")
            .help("Unique identifier for the store")
            .build()
            // Type
            .new_field("type")
            .readonly()
            .label("Type")
            .help("Storage backend type")
            .default("rocksdb")
            .typ(Type::Select {
                source: Source::Static(&[
                    ("rocksdb", "RocksDB"),
                    ("foundationdb", "FoundationDB"),
                    ("postgresql", "PostgreSQL"),
                    ("mysql", "mySQL"),
                    ("sqlite", "SQLite"),
                    ("s3", "S3-compatible"),
                    ("redis", "Redis/Memcached"),
                    ("elasticsearch", "ElasticSearch"),
                    ("azure", "Azure blob storage"),
                    ("fs", "Filesystem"),
                    ("sql-read-replica", "SQL with Replicas ⭐"),
                    ("distributed-blob", "Distributed Blob ⭐"),
                ]),
                typ: SelectType::Single,
            })
            .build()
            // Compression
            .new_field("compression")
            .readonly()
            .label("Compression")
            .help("Algorithm to use to compress large binary objects")
            .default("lz4")
            .typ(Type::Select {
                source: Source::Static(&[("none", "None"), ("lz4", "LZ4")]),
                typ: SelectType::Single,
            })
            .display_if_ne("type", ["redis", "memory", "elasticsearch"])
            .build()
            // Path
            .new_field("path")
            .label("Path")
            .help("Where to store the data in the server's filesystem")
            .display_if_eq("type", ["rocksdb", "sqlite", "fs"])
            .typ(Type::Input)
            .input_check([Transformer::Trim], [Validator::Required])
            .build()
            // Host
            .new_field("host")
            .label("Hostname")
            .help("Hostname of the database server")
            .display_if_eq("type", ["postgresql", "mysql"])
            .typ(Type::Input)
            .input_check(
                [Transformer::Trim],
                [Validator::Required, Validator::IsHost],
            )
            .build()
            // Port
            .new_field("port")
            .label("Port")
            .help("Port of the database server")
            .display_if_eq("type", ["postgresql", "mysql"])
            .default_if_eq("type", ["postgresql"], "5432")
            .default_if_eq("type", ["mysql"], "3307")
            .typ(Type::Input)
            .input_check(
                [Transformer::Trim],
                [Validator::Required, Validator::IsPort],
            )
            .build()
            // Database name
            .new_field("database")
            .label("Database")
            .help("Name of the database")
            .default("stalwart")
            .display_if_eq("type", ["postgresql", "mysql"])
            .typ(Type::Input)
            .input_check([Transformer::Trim], [Validator::Required])
            .build()
            // Redis type
            .new_field("redis-type")
            .label("Server Type")
            .help("Type of Redis server")
            .display_if_eq("type", ["redis"])
            .default("single")
            .typ(Type::Select {
                source: Source::Static(&[
                    ("single", "Redis single node"),
                    ("cluster", "Redis Cluster"),
                ]),
                typ: SelectType::Single,
            })
            .build()
            // Username
            .new_field("user")
            .label("Username")
            .help("Username to connect to the database")
            .default("stalwart")
            .display_if_eq("type", ["postgresql", "mysql", "elasticsearch"])
            .display_if_eq("redis-type", ["cluster"])
            .typ(Type::Input)
            .input_check([Transformer::Trim], [])
            .build()
            // Password
            .new_field("password")
            .label("Password")
            .help("Password to connect to the database")
            .display_if_eq("type", ["postgresql", "mysql", "elasticsearch"])
            .display_if_eq("redis-type", ["cluster"])
            .typ(Type::Secret)
            .build()
            // Timeout
            .new_field("timeout")
            .label("Timeout")
            .help("Connection timeout to the database")
            .display_if_eq("type", ["postgresql", "mysql", "redis", "s3", "azure"])
            .typ(Type::Duration)
            .default("15s")
            .build()
            // Purge frequency
            .new_field("purge.frequency")
            .label("Purge Frequency")
            .help("How often to purge the database. Expects a cron expression")
            .display_if_ne("type", ["redis", "memory", "elasticsearch"])
            .default("0 3 *")
            .typ(Type::Cron)
            .input_check([Transformer::Trim], [Validator::Required])
            .build()
            // Workers
            .new_field("pool.workers")
            .label("Thread Pool Size")
            .help("Number of worker threads to use for the store, defaults to the number of cores")
            .display_if_eq("type", ["rocksdb", "sqlite"])
            .placeholder("8")
            .typ(Type::Input)
            .input_check(
                [Transformer::Trim],
                [
                    Validator::MinValue(1.into()),
                    Validator::MaxValue(64.into()),
                ],
            )
            .build()
            // Number of connections
            .new_field("pool.max-connections")
            .label("Max Connections")
            .help("Maximum number of connections to the store")
            .display_if_eq("type", ["postgresql", "mysql", "sqlite"])
            .default("10")
            .typ(Type::Input)
            .input_check(
                [Transformer::Trim],
                [
                    Validator::MinValue(1.into()),
                    Validator::MaxValue(8192.into()),
                ],
            )
            .build()
            .new_field("pool.min-connections")
            .label("Min Connections")
            .help("Minimum number of connections to the store")
            .display_if_eq("type", ["mysql"])
            .default("5")
            .typ(Type::Input)
            .input_check(
                [Transformer::Trim],
                [
                    Validator::MinValue(1.into()),
                    Validator::MaxValue(8192.into()),
                ],
            )
            .build()
            // TLS
            .new_field("tls.enable")
            .label("Enable TLS")
            .help("Use TLS to connect to the store")
            .display_if_eq("type", ["postgresql", "mysql"])
            .default("false")
            .typ(Type::Boolean)
            .build()
            .new_field("tls.allow-invalid-certs")
            .label("Allow Invalid Certs")
            .help("Allow invalid TLS certificates when connecting to the store")
            .display_if_eq("type", ["postgresql", "mysql", "elasticsearch"])
            .default("false")
            .typ(Type::Boolean)
            .build()
            // URL
            .new_field("url")
            .label("URL")
            .help("URL of the store")
            .display_if_eq("type", ["elasticsearch"])
            .default_if_eq("type", ["elasticsearch"], "https://localhost:9200")
            .typ(Type::Input)
            .input_check([Transformer::Trim], [Validator::Required, Validator::IsUrl])
            .build()
            // Maximum number of retries
            .new_field("max-retries")
            .label("Retry limit")
            .help(concat!(
                "The maximum number of times to retry failed requests. ",
                "Set to 0 to disable retries"
            ))
            .display_if_eq("type", ["s3", "azure"])
            .placeholder("3")
            .default("3")
            .typ(Type::Input)
            .input_check(
                [Transformer::Trim],
                [
                    Validator::MinValue(1.into()),
                    Validator::MaxValue(10.into()),
                ],
            )
            .build()
            // Key prefix (for blob stores)
            .new_field("key-prefix")
            .label("Key Prefix")
            .help("A prefix that will be added to the keys of all objects stored in the blob store")
            .display_if_eq("type", ["s3", "azure"])
            .input_check([Transformer::Trim], [])
            .build()
            // SQL directory specific
            .new_field("query.name")
            .label("Account by Name")
            .help("Query to obtain the account details by login name")
            .typ(Type::Input)
            .input_check([Transformer::Trim], [])
            .display_if_eq("type", ["postgresql", "mysql", "sqlite"])
            .default_if_eq("type", ["postgresql"], PGSQL_NAME)
            .default_if_eq("type", ["mysql"], MYSQL_NAME)
            .default_if_eq("type", ["sqlite"], SQLITE_NAME)
            .new_field("query.members")
            .label("Members")
            .help("Query to obtain the members of a group by account name")
            .default_if_eq("type", ["postgresql"], PGSQL_MEMBERS)
            .default_if_eq("type", ["mysql"], MYSQL_MEMBERS)
            .default_if_eq("type", ["sqlite"], SQLITE_MEMBERS)
            .new_field("query.recipients")
            .label("Recipients")
            .help("Query to obtain the recipient(s) of an e-mail address")
            .default_if_eq("type", ["postgresql"], PGSQL_RECIPIENTS)
            .default_if_eq("type", ["mysql"], MYSQL_RECIPIENTS)
            .default_if_eq("type", ["sqlite"], SQLITE_RECIPIENTS)
            .new_field("query.emails")
            .label("E-mails")
            .help("Query to obtain the e-mail address(es) of an account")
            .default_if_eq("type", ["postgresql"], PGSQL_EMAILS)
            .default_if_eq("type", ["mysql"], MYSQL_EMAILS)
            .default_if_eq("type", ["sqlite"], SQLITE_EMAILS)
            .new_field("query.verify")
            .label("Verify (VRFY)")
            .help("Query to verify an e-mail address with the VRFY SMTP command")
            .default_if_eq("type", ["postgresql"], PGSQL_VRFY)
            .default_if_eq("type", ["mysql"], MYSQL_VRFY)
            .default_if_eq("type", ["sqlite"], SQLITE_VRFY)
            .new_field("query.expand")
            .label("Expand (EXPN)")
            .help("Query to expand an e-mail address with the EXPN SMTP command")
            .default_if_eq("type", ["postgresql"], PGSQL_EXPN)
            .default_if_eq("type", ["mysql"], MYSQL_EXPN)
            .default_if_eq("type", ["sqlite"], SQLITE_EXPN)
            .new_field("query.domains")
            .label("Local domains")
            .help("Query to verify whether a domain is local")
            .default_if_eq("type", ["postgresql"], PGSQL_DOMAINS)
            .default_if_eq("type", ["mysql"], MYSQL_DOMAINS)
            .default_if_eq("type", ["sqlite"], SQLITE_DOMAINS)
            .build()
            // RocksDB specific
            .new_field("settings.min-blob-size")
            .label("Min blob size")
            .help(concat!(
                "Minimum size of a blob to store in the blob store, ",
                "smaller blobs are stored in the metadata store"
            ))
            .display_if_eq("type", ["rocksdb"])
            .default("16834")
            .typ(Type::Size)
            .input_check(
                [Transformer::Trim],
                [
                    Validator::MinValue(1024.into()),
                    Validator::MaxValue((1024 * 1024).into()),
                ],
            )
            .new_field("settings.write-buffer-size")
            .label("Write buffer size")
            .help("Size of the write buffer in bytes, used to batch writes to the store")
            .default("134217728")
            .typ(Type::Size)
            .input_check(
                [Transformer::Trim],
                [
                    Validator::MinValue(8192.into()),
                    Validator::MaxValue((1024 * 1024 * 1024).into()),
                ],
            )
            .build()
            // FoundationDB specific
            .new_field("cluster-file")
            .label("Cluster file")
            .help("Path to the cluster file for the FoundationDB cluster")
            .display_if_eq("type", ["foundationdb"])
            .placeholder("/etc/foundationdb/fdb.cluster")
            .typ(Type::Input)
            .input_check([Transformer::Trim], [])
            .new_field("transaction.timeout")
            .label("Timeout")
            .help("Transaction timeout")
            .placeholder("5s")
            .typ(Type::Duration)
            .new_field("transaction.max-retry-delay")
            .label("Max Retry Delay")
            .help("Transaction maximum retry delay")
            .placeholder("1s")
            .typ(Type::Duration)
            .new_field("transaction.retry-limit")
            .label("Retry limit")
            .help("Transaction retry limit")
            .placeholder("10")
            .typ(Type::Input)
            .input_check(
                [Transformer::Trim],
                [
                    Validator::MinValue(1.into()),
                    Validator::MaxValue(1000.into()),
                ],
            )
            .new_field("ids.machine")
            .label("Machine Id")
            .help("Machine ID in the FoundationDB cluster (optional)")
            .placeholder("my-server-id")
            .typ(Type::Input)
            .input_check([Transformer::Trim], [Validator::IsId])
            .new_field("ids.data-center")
            .label("Data Center Id")
            .help("Data center ID (optional)")
            .placeholder("my-datacenter-id")
            .build()
            // mySQL specific
            .new_field("max-allowed-packet")
            .label("Max Allowed Packet")
            .help("Maximum size of a packet in bytes")
            .display_if_eq("type", ["mysql"])
            .placeholder("1073741824")
            .typ(Type::Size)
            .input_check(
                [Transformer::Trim],
                [
                    Validator::MinValue(1024.into()),
                    Validator::MaxValue((1024 * 1024 * 1024).into()),
                ],
            )
            .build()
            // ElasticSearch specific
            .new_field("cloud-id")
            .label("Cloud Id")
            .help("Cloud ID for the ElasticSearch cluster")
            .display_if_eq("type", ["elasticsearch"])
            .placeholder("my-cloud-id")
            .typ(Type::Input)
            .input_check([Transformer::Trim], [])
            .new_field("index.shards")
            .label("Number of Shards")
            .help("Number of shards for the index")
            .default("3")
            .typ(Type::Input)
            .input_check(
                [Transformer::Trim],
                [
                    Validator::MinValue(1.into()),
                    Validator::MaxValue((1024 * 1024).into()),
                ],
            )
            .new_field("index.replicas")
            .label("Number of Replicas")
            .help("Number of replicas for the index")
            .default("0")
            .typ(Type::Input)
            .input_check(
                [Transformer::Trim],
                [
                    Validator::MinValue(0.into()),
                    Validator::MaxValue(2048.into()),
                ],
            )
            .build()
            // Redis specific
            .new_field("urls")
            .label("URL(s)")
            .help("URL(s) of the Redis server(s)")
            .display_if_eq("type", ["redis"])
            .default("redis://127.0.0.1")
            .typ(Type::Array)
            .input_check([Transformer::Trim], [Validator::Required, Validator::IsUrl])
            .build()
            .new_field("retry.total")
            .label("Retries")
            .help("Number of retries to connect to the Redis cluster")
            .display_if_eq("redis-type", ["cluster"])
            .placeholder("3")
            .typ(Type::Input)
            .input_check(
                [Transformer::Trim],
                [
                    Validator::MinValue(1.into()),
                    Validator::MaxValue(1024.into()),
                ],
            )
            .new_field("retry.max-wait")
            .label("Max Wait")
            .help("Maximum time to wait between retries")
            .placeholder("1s")
            .typ(Type::Duration)
            .new_field("retry.min-wait")
            .label("Min Wait")
            .help("Minimum time to wait between retries")
            .placeholder("500ms")
            .build()
            .new_field("read-from-replicas")
            .label("Read from replicas")
            .help("Whether to read from replicas")
            .default("true")
            .typ(Type::Boolean)
            .build()
            // S3 specific
            .new_field("bucket")
            .typ(Type::Input)
            .label("Name")
            .help("The S3 bucket where blobs (e-mail messages, Sieve scripts, etc.) will be stored")
            .input_check([Transformer::Trim], [Validator::Required])
            .placeholder("stalwart")
            .display_if_eq("type", ["s3"])
            .new_field("region")
            .label("Region")
            .help("The geographical region where the bucket resides")
            .placeholder("us-east-1")
            .new_field("endpoint")
            .help(concat!(
                "The network address (hostname and optionally a port) of the S3 service. ",
                "If you are using a well-known S3 service like Amazon S3, this setting can ",
                "be left blank, and the endpoint will be derived from the region. For ",
                "S3-compatible services, you will need to specify the endpoint explicitly"
            ))
            .label("Endpoint")
            .new_field("profile")
            .label("Profile")
            .help(concat!(
                "Used when retrieving credentials from a shared credentials file. If specified, ",
                "the server will use the access key ID, secret access key, and session token (if ",
                "available) associated with the given profile"
            ))
            .new_field("access-key")
            .label("Access Key")
            .help("Identifies the S3 account")
            .new_field("secret-key")
            .label("Secret Key")
            .help("The secret key for the S3 account")
            .typ(Type::Secret)
            .new_field("security-token")
            .label("Security Token")
            .build()
            // Azure specific
            .new_field("storage-account")
            .typ(Type::Input)
            .label("Storage Account Name")
            .help("The Azure Storage Account where blobs (e-mail messages, Sieve scripts, etc.) will be stored")
            .input_check([Transformer::Trim], [Validator::Required])
            .placeholder("mycompany")
            .display_if_eq("type", ["azure"])
            .new_field("container")
            .typ(Type::Input)
            .label("Container")
            .help("The name of the container in the Storage Account")
            .input_check([Transformer::Trim], [Validator::Required])
            .placeholder("stalwart")
            .new_field("azure-access-key")
            .label("Access Key")
            .help("The access key for the Azure Storage Account")
            .typ(Type::Secret)
            .input_check([Transformer::Trim], [])
            .new_field("sas-token")
            .label("SAS Token (if not using access-key based authentication)")
            .typ(Type::Secret)
            .input_check([Transformer::Trim], [])
            .build()
            // FS specific
            .new_field("depth")
            .label("Nested Depth")
            .help("Maximum depth of nested directories")
            .display_if_eq("type", ["fs"])
            .default("2")
            .typ(Type::Input)
            .input_check(
                [Transformer::Trim],
                [Validator::MinValue(0.into()), Validator::MaxValue(5.into())],
            )
            .build()
            // SQL read replicas
            .new_field("primary")
            .label("Primary SQL")
            .help("Primary SQL store where the data is written")
            .display_if_eq("type", ["sql-read-replica"])
            .typ(Type::Select {
                source: Source::DynamicSelf {
                    field: "type",
                    filter: Default::default(),
                },
                typ: SelectType::Single,
            })
            .source_filter(&["mysql", "postgresql"])
            .input_check([], [Validator::Required])
            .build()
            .new_field("replicas")
            .label("Read replicas")
            .help("The read replicas where the data is read from")
            .display_if_eq("type", ["sql-read-replica"])
            .typ(Type::Select {
                source: Source::DynamicSelf {
                    field: "type",
                    filter: Default::default(),
                },
                typ: SelectType::ManyWithSearch,
            })
            .source_filter(&["mysql", "postgresql"])
            .input_check([], [Validator::Required])
            .build()
            // Distributed blobs
            .new_field("stores")
            .label("Blob stores")
            .help("Blob stores to use for the distributed blob store")
            .display_if_eq("type", ["distributed-blob"])
            .typ(Type::Select {
                source: Source::DynamicSelf {
                    field: "type",
                    filter: Default::default(),
                },
                typ: SelectType::ManyWithSearch,
            })
            .source_filter(&["s3", "fs"])
            .input_check([], [Validator::Required])
            .build()
            // Form layouts
            .new_form_section()
            .title("Configuration")
            .fields([
                "_id",
                "type",
                "path",
                "cluster-file",
                "redis-type",
                "host",
                "port",
                "database",
                "url",
                "urls",
                "max-allowed-packet",
                "region",
                "endpoint",
                "cloud-id",
                "profile",
                "timeout",
                "primary",
                "replicas",
                "stores",
            ])
            .build()
            .new_form_section()
            .title("Bucket")
            .display_if_eq("type", ["s3"])
            .fields(["bucket", "key-prefix"])
            .build()
            .new_form_section()
            .title("Storage Account")
            .display_if_eq("type", ["azure"])
            .fields(["storage-account", "container", "key-prefix"])
            .build()
            .new_form_section()
            .title("Authentication")
            .display_if_eq("type", ["postgresql", "mysql", "elasticsearch", "s3", "azure"])
            .display_if_eq("redis-type", ["cluster"])
            .fields([
                "user",
                "password",
                "access-key",
                "secret-key",
                "security-token",
                "azure-access-key",
                "sas-token",
            ])
            .build()
            .new_form_section()
            .title("Storage settings")
            .display_if_eq(
                "type",
                [
                    "postgresql",
                    "mysql",
                    "sqlite",
                    "rocksdb",
                    "foundationdb",
                    "fs",
                    "s3",
                    "azure",
                    "sql-read-replica",
                    "distributed-blob",
                ],
            )
            .fields([
                "compression",
                "settings.min-blob-size",
                "settings.write-buffer-size",
                "max-retries",
                "depth",
                "purge.frequency",
            ])
            .build()
            .new_form_section()
            .title("TLS")
            .display_if_eq("type", ["postgresql", "mysql", "elasticsearch"])
            .fields(["tls.enable", "tls.allow-invalid-certs"])
            .build()
            .new_form_section()
            .title("Pools")
            .display_if_eq("type", ["rocksdb", "sqlite", "postgresql", "mysql"])
            .fields([
                "pool.workers",
                "pool.max-connections",
                "pool.min-connections",
            ])
            .build()
            .new_form_section()
            .title("Cluster Settings")
            .display_if_eq("redis-type", ["cluster"])
            .fields([
                "read-from-replicas",
                "retry.total",
                "retry.max-wait",
                "retry.min-wait",
            ])
            .build()
            .new_form_section()
            .title("Cluster Ids")
            .display_if_eq("type", ["foundationdb"])
            .fields(["ids.machine", "ids.data-center"])
            .build()
            .new_form_section()
            .title("Transaction Settings")
            .display_if_eq("type", ["foundationdb"])
            .fields([
                "transaction.timeout",
                "transaction.max-retry-delay",
                "transaction.retry-limit",
            ])
            .build()
            .new_form_section()
            .title("Directory Queries")
            .display_if_eq("type", ["postgresql", "mysql", "sqlite"])
            .fields([
                "query.name",
                "query.members",
                "query.recipients",
                "query.emails",
                "query.verify",
                "query.expand",
                "query.domains",
            ])
            .build()
            .new_form_section()
            .title("Index")
            .display_if_eq("type", ["elasticsearch"])
            .fields(["index.shards", "index.replicas"])
            .build()
            .list_title("Stores")
            .list_subtitle("Manage data, blob, full-text, and lookup stores")
            .list_fields(["_id", "type"])
            .build()
    }
}
