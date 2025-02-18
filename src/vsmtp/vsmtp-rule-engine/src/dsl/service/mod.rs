/*
 * vSMTP mail transfer agent
 * Copyright (C) 2022 viridIT SAS
 *
 * This program is free software: you can redistribute it and/or modify it under
 * the terms of the GNU General Public License as published by the Free Software
 * Foundation, either version 3 of the License, or any later version.
 *
 * This program is distributed in the hope that it will be useful, but WITHOUT
 * ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS
 * FOR A PARTICULAR PURPOSE.  See the GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License along with
 * this program. If not, see https://www.gnu.org/licenses/.
 *
*/

use vsmtp_common::transfer::SmtpConnection;

use crate::api::EngineResult;

pub mod cmd;
pub mod databases;
pub mod parsing;
pub mod smtp;

/// service that enable the user to integrate third party software
/// into his rules.
#[derive(Debug)]
pub enum Service {
    /// A service can be a program to run in a subprocess
    Cmd {
        /// A duration after which the subprocess will be forced-kill
        timeout: std::time::Duration,
        /// Optional: a user to run the subprocess under
        user: Option<String>,
        /// Optional: a group to run the subprocess under
        group: Option<String>,
        /// The command to execute in the subprocess
        command: String,
        /// Optional: parameters directly given to the executed program (argc, argv)
        args: Option<Vec<String>>,
    },

    /// A database connector based on the csv file format.
    CSVDatabase {
        /// A path to the file to open.
        path: std::path::PathBuf,
        /// Access mode to the database.
        access: databases::AccessMode,
        /// Delimiter character to separate fields in records.
        delimiter: u8,
        /// Database refresh mode.
        refresh: databases::Refresh,
        /// Raw content of the database.
        fd: std::fs::File,
    },

    #[cfg(feature = "mysql")]
    /// A database connector based on MySQL.
    MySQLDatabase {
        /// The url to the database.
        url: String,
        /// connection pool for the database.
        pool: r2d2::Pool<self::databases::mysql::MySQLConnectionManager>,
    },

    /// A service that handles smtp transactions.
    Smtp {
        /// A transport to handle transactions to the delegate.
        delegator: SmtpConnection,
        /// Delegation results address.
        receiver: std::net::SocketAddr,
    },
}

impl std::fmt::Display for Service {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Service::Cmd { .. } => "cmd",
                Self::Smtp { .. } => "smtp",
                Self::CSVDatabase { .. } => "csv-database",
                #[cfg(feature = "mysql")]
                Self::MySQLDatabase { .. } => "mysql-database",
            }
        )
    }
}

/// extract a value from a `rhai::Map`, optionally inserting a default value.
pub fn get_or_default<T: Clone + Send + Sync + 'static>(
    map_name: &str,
    map: &rhai::Map,
    key: &str,
    default: Option<T>,
) -> EngineResult<T> {
    fn try_cast<T: Clone + Send + Sync + 'static>(
        name: &str,
        value: &rhai::Dynamic,
    ) -> EngineResult<T> {
        value
            .clone()
            .try_cast::<T>()
            .ok_or_else::<Box<rhai::EvalAltResult>, _>(|| {
                format!(
                    "the {name} parameter for a smtp service must be a {}",
                    std::any::type_name::<T>()
                )
                .into()
            })
    }

    match (map.get(key), default) {
        (Some(value), _) => try_cast(key, value),
        (mut value, Some(default)) => {
            try_cast(key, value.get_or_insert(&rhai::Dynamic::from(default)))
        }
        _ => Err(format!("key {key} was not found in {map_name}").into()),
    }
}
