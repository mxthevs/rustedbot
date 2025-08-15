use rusqlite::params_from_iter;
use rusqlite::Connection;

pub fn migrate(trusted_users: Vec<String>) {
    let connection = Connection::open("./database/rusted.db").unwrap();

    let create_commands_table_query = "
      CREATE TABLE IF NOT EXISTS commands (
        id INTEGER PRIMARY KEY,
        name TEXT UNIQUE NOT NULL,
        response TEXT NOT NULL,
        created_at DATETIME NOT NULL,
        updated_at DATETIME NOT NULL,
        deleted_at DATETIME
      )
    ";

    connection.execute(create_commands_table_query, []).unwrap();

    let create_trusted_users_table_query = "
      CREATE TABLE IF NOT EXISTS trusted_users (
        id INTEGER PRIMARY KEY,
        username TEXT UNIQUE NOT NULL,
        created_at DATETIME NOT NULL,
        updated_at DATETIME NOT NULL,
        deleted_at DATETIME
      )
    ";

    connection
        .execute(create_trusted_users_table_query, [])
        .unwrap();

    if trusted_users.is_empty() {
        return ();
    }

    let values = (1..=trusted_users.len())
        .map(|i| format!("(?{i}, datetime('now'), datetime('now'))"))
        .collect::<Vec<_>>()
        .join(", ");

    let insert_trusted_users_query = format!(
        "
          INSERT INTO trusted_users (username, created_at, updated_at)
          VALUES {values}
          ON CONFLICT(username) DO NOTHING
          "
    );

    connection
        .execute(
            &insert_trusted_users_query,
            params_from_iter(trusted_users.iter().map(|user| user.as_str())),
        )
        .unwrap();
}

pub fn create_command(name: &str, response: &str) {
    let connection = Connection::open("./database/rusted.db").unwrap();

    let create_command_query = "
      INSERT INTO commands (name, response, created_at, updated_at)
      VALUES (?1, ?2, datetime('now'), datetime('now'))
    ";

    connection
        .execute(create_command_query, &[&name, &response])
        .unwrap();
}

pub fn get_commands() -> Result<Vec<(String, String)>, rusqlite::Error> {
    let connection = Connection::open("./database/rusted.db").unwrap();

    let get_commands_query = "
      SELECT name, response
      FROM commands
      WHERE deleted_at IS NULL
    ";

    let mut statement = connection.prepare(get_commands_query).unwrap();
    let commands = statement.query_map([], |row| Ok((row.get(0).unwrap(), row.get(1).unwrap())))?;

    let mut result = Vec::new();
    for command in commands {
        result.push(command?);
    }

    Ok(result)
}

pub fn get_command_response(name: &str) -> Result<String, rusqlite::Error> {
    let connection = Connection::open("./database/rusted.db").unwrap();

    let get_command_response_query = "
      SELECT response
      FROM commands
      WHERE name = ?
      AND deleted_at IS NULL
    ";

    let mut statement = connection.prepare(get_command_response_query).unwrap();
    let responses = statement.query_map(&[&name], |row| Ok(row.get(0).unwrap()));

    match responses {
        Ok(responses) => {
            for response in responses {
                return Ok(response.unwrap());
            }
        }
        Err(e) => return Err(e),
    }

    Err(rusqlite::Error::QueryReturnedNoRows)
}

pub fn update_command_response(name: &str, response: &str) {
    let connection = Connection::open("./database/rusted.db").unwrap();

    let update_command_response_query = "
      UPDATE commands
      SET response = ?, updated_at = datetime('now')
      WHERE name = ?
    ";

    connection
        .execute(update_command_response_query, &[&response, &name])
        .unwrap();
}

pub fn delete_command(name: &str) {
    let connection = Connection::open("./database/rusted.db").unwrap();

    let delete_command_query = "
      UPDATE commands
      SET deleted_at = datetime('now')
      WHERE name = ?
    ";

    connection.execute(delete_command_query, &[&name]).unwrap();
}

pub fn is_trusted(username: &str) -> bool {
    let connection = Connection::open("./database/rusted.db").unwrap();

    let is_trusted_query = "
      SELECT 1
      FROM trusted_users
      WHERE username = ?
      AND deleted_at IS NULL
      LIMIT 1
    ";

    let result = connection.query_row(is_trusted_query, rusqlite::params![username], |row| {
        row.get::<usize, i32>(0)
    });

    result.is_ok()
}

pub fn trust_user(username: &str) {
    let connection = Connection::open("./database/rusted.db").unwrap();

    if is_trusted(username) {
        return ();
    }

    let trust_user_query = "
      INSERT INTO trusted_users (username, created_at, updated_at)
      VALUES (?, datetime('now'), datetime('now'))
      ON CONFLICT(username) DO NOTHING
    ";

    connection.execute(trust_user_query, &[&username]).unwrap();
}

pub fn untrust_user(username: &str) {
    let connection = Connection::open("./database/rusted.db").unwrap();

    let untrust_user_query = "
      UPDATE trusted_users
      SET deleted_at = datetime('now')
      WHERE username = ?
    ";

    connection
        .execute(untrust_user_query, &[&username])
        .unwrap();
}
