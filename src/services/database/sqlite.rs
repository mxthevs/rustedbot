use rusqlite::{params_from_iter, Connection, Result};

const DATABASE_PATH: &str = "./database/rusted.db";

pub fn migrate(trusted_users: Vec<String>) -> Result<()> {
    let connection = Connection::open(DATABASE_PATH)?;

    const CREATE_COMMANDS_TABLE: &str = "
		CREATE TABLE IF NOT EXISTS commands (
			id INTEGER PRIMARY KEY,
			name TEXT UNIQUE NOT NULL,
			response TEXT NOT NULL,
			created_at DATETIME NOT NULL,
			updated_at DATETIME NOT NULL,
			deleted_at DATETIME
		)
	";

    const CREATE_TRUSTED_USERS_TABLE: &str = "
		CREATE TABLE IF NOT EXISTS trusted_users (
			id INTEGER PRIMARY KEY,
			username TEXT UNIQUE NOT NULL,
			created_at DATETIME NOT NULL,
			updated_at DATETIME NOT NULL,
			deleted_at DATETIME
		)
	";

    create_table(&connection, CREATE_COMMANDS_TABLE)?;
    create_table(&connection, CREATE_TRUSTED_USERS_TABLE)?;

    if !trusted_users.is_empty() {
        insert_trusted_users(&connection, &trusted_users)?;
    }

    Ok(())
}

fn create_table(conn: &Connection, ddl: &str) -> Result<()> {
    conn.execute(ddl, [])?;
    Ok(())
}

fn insert_trusted_users(conn: &Connection, users: &[String]) -> Result<()> {
    let placeholders = (1..=users.len())
        .map(|i| format!("(?{i}, datetime('now'), datetime('now'))"))
        .collect::<Vec<_>>()
        .join(", ");

    let query = format!(
        "
        INSERT INTO trusted_users (username, created_at, updated_at)
        VALUES {placeholders}
        ON CONFLICT(username) DO UPDATE SET
            deleted_at = NULL,
            updated_at = datetime('now')
        "
    );

    conn.execute(&query, params_from_iter(users.iter().map(|u| u.as_str())))?;
    Ok(())
}

pub fn create_command(name: &str, response: &str) {
    let connection = Connection::open(DATABASE_PATH).unwrap();

    const CREATE_COMMAND_QUERY: &str = "
    	INSERT INTO commands (name, response, created_at, updated_at)
    	VALUES (?1, ?2, datetime('now'), datetime('now'))
    ";

    connection
        .execute(CREATE_COMMAND_QUERY, [&name, &response])
        .unwrap();
}

pub fn get_commands() -> Result<Vec<(String, String)>, rusqlite::Error> {
    let connection = Connection::open(DATABASE_PATH)?;

    const GET_COMMANDS_QUERY: &str = "
     	SELECT name, response
    	FROM commands
    	WHERE deleted_at IS NULL
    ";

    let mut statement = connection.prepare(GET_COMMANDS_QUERY)?;
    let commands = statement.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?;

    let mut result = Vec::new();
    for command in commands {
        result.push(command?);
    }

    Ok(result)
}

pub fn get_command_response(name: &str) -> Result<String, rusqlite::Error> {
    let connection = Connection::open(DATABASE_PATH)?;

    const GET_COMMAND_RESPONSE_QUERY: &str = "
    	SELECT response
    	FROM commands
    	WHERE name = ?
    	AND deleted_at IS NULL
    ";

    let mut statement = connection.prepare(GET_COMMAND_RESPONSE_QUERY)?;
    let responses = statement.query_map([&name], |row| Ok(row.get(0)?));

    match responses {
        Ok(mut responses) => {
            if let Some(response) = responses.next() {
                return Ok(response?);
            }
        }
        Err(e) => return Err(e),
    }

    Err(rusqlite::Error::QueryReturnedNoRows)
}

pub fn update_command_response(name: &str, response: &str) {
    let connection = Connection::open(DATABASE_PATH).unwrap();

    const UPDATE_COMMAND_RESPONSE_QUERY: &str = "
    	UPDATE commands
    	SET response = ?, updated_at = datetime('now')
    	WHERE name = ?
    ";

    connection
        .execute(UPDATE_COMMAND_RESPONSE_QUERY, [&response, &name])
        .unwrap();
}

pub fn delete_command(name: &str) {
    let connection = Connection::open(DATABASE_PATH).unwrap();

    const DELETE_COMMAND_QUERY: &str = "
    	UPDATE commands
    	SET deleted_at = datetime('now')
    	WHERE name = ?
    ";

    connection.execute(DELETE_COMMAND_QUERY, [&name]).unwrap();
}

pub fn is_trusted(username: &str) -> bool {
    let connection = Connection::open(DATABASE_PATH).unwrap();

    const IS_TRUSTED_QUERY: &str = "
    	SELECT 1
    	FROM trusted_users
    	WHERE username = ?
    	AND deleted_at IS NULL
    	LIMIT 1
	";

    let result = connection.query_row(IS_TRUSTED_QUERY, rusqlite::params![username], |row| {
        row.get::<usize, i32>(0)
    });

    result.is_ok()
}

pub fn trust_user(username: &str) {
    let connection = Connection::open(DATABASE_PATH).unwrap();

    if is_trusted(username) {
        return;
    }

    const TRUST_USER_QUERY: &str = "
        INSERT INTO trusted_users (username, created_at, updated_at)
        VALUES (?, datetime('now'), datetime('now'))
        ON CONFLICT(username) DO UPDATE SET
            deleted_at = NULL,
            updated_at = datetime('now')
    ";

    connection.execute(TRUST_USER_QUERY, [&username]).unwrap();
}

pub fn untrust_user(username: &str) {
    let connection = Connection::open(DATABASE_PATH).unwrap();

    const UNTRUST_USER_QUERY: &str = "
    	UPDATE trusted_users
    	SET deleted_at = datetime('now'),
			updated_at = datetime('now')
    	WHERE username = ?
    ";

    connection.execute(UNTRUST_USER_QUERY, [&username]).unwrap();
}
