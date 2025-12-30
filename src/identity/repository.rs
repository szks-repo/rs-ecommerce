use sqlx::Row;
use sqlx::{Executor, Postgres};

use crate::identity::error::{IdentityError, IdentityResult};

pub struct PgIdentityRepository<'a> {
    db: &'a sqlx::PgPool,
}

pub struct StaffSummaryRow {
    pub staff_id: String,
    pub email: Option<String>,
    pub login_id: Option<String>,
    pub phone: Option<String>,
    pub status: String,
    pub display_name: Option<String>,
    pub role_id: String,
    pub role_key: String,
}

pub struct StaffAuthRow {
    pub staff_id: String,
    pub password_hash: Option<String>,
    pub role_key: String,
}

pub struct RoleRow {
    pub id: String,
    pub key: String,
    pub name: Option<String>,
}

pub struct RoleDetailRow {
    pub id: String,
    pub key: String,
    pub name: String,
    pub description: Option<String>,
}

pub trait IdentityRepository {
    /// list_staff
    async fn list_staff(&self, store_uuid: &uuid::Uuid) -> IdentityResult<Vec<StaffSummaryRow>>;

    /// staff_role_key
    async fn staff_role_key(
        &self,
        store_uuid: &uuid::Uuid,
        staff_uuid: &uuid::Uuid,
    ) -> IdentityResult<Option<String>>;

    /// role_by_id
    async fn role_by_id(
        &self,
        store_uuid: &uuid::Uuid,
        role_uuid: &uuid::Uuid,
    ) -> IdentityResult<Option<RoleRow>>;

    /// role_key_by_id
    async fn role_key_by_id(
        &self,
        store_uuid: &uuid::Uuid,
        role_id: &str,
    ) -> IdentityResult<Option<String>>;

    /// role_store_id
    async fn role_store_id(&self, role_id: &str) -> IdentityResult<Option<String>>;

    /// update_staff
    async fn update_staff(
        &self,
        staff_uuid: &uuid::Uuid,
        store_uuid: &uuid::Uuid,
        role_id: &str,
        status: &str,
        display_name: &str,
    ) -> IdentityResult<Option<StaffSummaryRow>>;

    /// insert_staff
    async fn insert_staff(
        &self,
        staff_id: &uuid::Uuid,
        store_uuid: &uuid::Uuid,
        email: Option<&str>,
        login_id: Option<&str>,
        phone: Option<&str>,
        password_hash: &str,
        role_uuid: &uuid::Uuid,
        status: &str,
        display_name: Option<&str>,
    ) -> IdentityResult<()>;

    async fn staff_summary(
        &self,
        store_uuid: &uuid::Uuid,
        staff_uuid: &uuid::Uuid,
    ) -> IdentityResult<Option<StaffSummaryRow>>;

    async fn store_name(&self, store_uuid: &uuid::Uuid) -> IdentityResult<Option<String>>;

    async fn role_id_by_key(
        &self,
        store_uuid: &uuid::Uuid,
        role_key: &str,
    ) -> IdentityResult<Option<uuid::Uuid>>;

    async fn list_roles(&self, store_uuid: &uuid::Uuid) -> IdentityResult<Vec<RoleDetailRow>>;

    async fn list_role_permissions(
        &self,
        role_ids: &[uuid::Uuid],
    ) -> IdentityResult<Vec<(String, String)>>;

    async fn permissions_by_keys(&self, keys: &[String]) -> IdentityResult<Vec<(String, String)>>;

    async fn role_attached(&self, role_uuid: &uuid::Uuid) -> IdentityResult<bool>;

    async fn delete_role(
        &self,
        store_uuid: &uuid::Uuid,
        role_uuid: &uuid::Uuid,
    ) -> IdentityResult<Option<RoleRow>>;

    async fn update_staff_role(
        &self,
        staff_uuid: &uuid::Uuid,
        store_uuid: &uuid::Uuid,
        role_uuid: &uuid::Uuid,
    ) -> IdentityResult<()>;

    async fn store_staff_exists_by_email(
        &self,
        store_uuid: &uuid::Uuid,
        email: &str,
    ) -> IdentityResult<bool>;

    async fn invite_exists_by_email(
        &self,
        store_uuid: &uuid::Uuid,
        email: &str,
    ) -> IdentityResult<bool>;

    async fn insert_staff_invite_tx<'e, E>(
        &self,
        exec: &mut E,
        staff_id: &uuid::Uuid,
        invite_id: &uuid::Uuid,
        store_uuid: &uuid::Uuid,
        email: &str,
        role_uuid: &uuid::Uuid,
        token: &str,
        created_by: Option<uuid::Uuid>,
        expires_at: chrono::DateTime<chrono::Utc>,
        display_name: Option<&str>,
    ) -> IdentityResult<()>
    where
        for<'c> &'c mut E: Executor<'c, Database = Postgres>;

    async fn current_owner_id(&self, store_uuid: &uuid::Uuid) -> IdentityResult<Option<String>>;

    async fn staff_status(
        &self,
        store_uuid: &uuid::Uuid,
        staff_uuid: &uuid::Uuid,
    ) -> IdentityResult<Option<String>>;

    async fn fetch_active_staff_by_email(
        &self,
        store_uuid: &uuid::Uuid,
        email: &str,
    ) -> IdentityResult<Option<StaffAuthRow>>;

    async fn fetch_active_staff_by_login_id(
        &self,
        store_uuid: &uuid::Uuid,
        login_id: &str,
    ) -> IdentityResult<Option<StaffAuthRow>>;

    async fn fetch_active_staff_by_phone(
        &self,
        store_uuid: &uuid::Uuid,
        phone: &str,
    ) -> IdentityResult<Option<StaffAuthRow>>;
}

impl<'a> IdentityRepository for PgIdentityRepository<'a> {
    async fn list_staff(&self, store_uuid: &uuid::Uuid) -> IdentityResult<Vec<StaffSummaryRow>> {
        let rows = sqlx::query(
            r#"
            SELECT ss.id::text as staff_id, ss.email, ss.login_id, ss.phone, ss.status,
                   ss.display_name, sr.id::text as role_id, sr.key as role_key
            FROM store_staff ss
            JOIN store_roles sr ON sr.id = ss.role_id
            WHERE ss.store_id = $1
            ORDER BY ss.created_at ASC
            "#,
        )
        .bind(store_uuid)
        .fetch_all(self.db)
        .await
        .map_err(IdentityError::from)?;

        Ok(rows
            .into_iter()
            .map(|row| StaffSummaryRow {
                staff_id: row.get("staff_id"),
                email: row.get("email"),
                login_id: row.get("login_id"),
                phone: row.get("phone"),
                status: row.get("status"),
                display_name: row.get("display_name"),
                role_id: row.get("role_id"),
                role_key: row.get("role_key"),
            })
            .collect())
    }

    async fn staff_role_key(
        &self,
        store_uuid: &uuid::Uuid,
        staff_uuid: &uuid::Uuid,
    ) -> IdentityResult<Option<String>> {
        let row = sqlx::query(
            r#"
            SELECT sr.key as role_key
            FROM store_staff ss
            JOIN store_roles sr ON sr.id = ss.role_id
            WHERE ss.id = $1 AND ss.store_id = $2
            "#,
        )
        .bind(staff_uuid)
        .bind(store_uuid)
        .fetch_optional(self.db)
        .await
        .map_err(IdentityError::from)?;
        Ok(row.map(|row| row.get("role_key")))
    }

    async fn role_by_id(
        &self,
        store_uuid: &uuid::Uuid,
        role_uuid: &uuid::Uuid,
    ) -> IdentityResult<Option<RoleRow>> {
        let row = sqlx::query(
            r#"
            SELECT id::text as id, key, name
            FROM store_roles
            WHERE id = $1 AND store_id = $2
            "#,
        )
        .bind(role_uuid)
        .bind(store_uuid)
        .fetch_optional(self.db)
        .await
        .map_err(IdentityError::from)?;
        Ok(row.map(|row| RoleRow {
            id: row.get("id"),
            key: row.get("key"),
            name: row.get("name"),
        }))
    }

    async fn update_staff(
        &self,
        staff_uuid: &uuid::Uuid,
        store_uuid: &uuid::Uuid,
        role_id: &str,
        status: &str,
        display_name: &str,
    ) -> IdentityResult<Option<StaffSummaryRow>> {
        let row = sqlx::query(
            r#"
            UPDATE store_staff
            SET role_id = COALESCE(NULLIF($1, '')::uuid, role_id),
                status = COALESCE(NULLIF($2, ''), status),
                display_name = COALESCE(NULLIF($3, ''), display_name),
                updated_at = now()
            WHERE id = $4 AND store_id = $5
            RETURNING id::text as staff_id, email, login_id, phone, status, role_id::text as role_id, display_name
            "#,
        )
        .bind(role_id)
        .bind(status)
        .bind(display_name)
        .bind(staff_uuid)
        .bind(store_uuid)
        .fetch_optional(self.db)
        .await
        .map_err(IdentityError::from)?;

        Ok(row.map(|row| StaffSummaryRow {
            staff_id: row.get("staff_id"),
            email: row.get("email"),
            login_id: row.get("login_id"),
            phone: row.get("phone"),
            status: row.get("status"),
            display_name: row.get("display_name"),
            role_id: row.get("role_id"),
            role_key: String::new(),
        }))
    }

    async fn insert_staff(
        &self,
        staff_id: &uuid::Uuid,
        store_uuid: &uuid::Uuid,
        email: Option<&str>,
        login_id: Option<&str>,
        phone: Option<&str>,
        password_hash: &str,
        role_uuid: &uuid::Uuid,
        status: &str,
        display_name: Option<&str>,
    ) -> IdentityResult<()> {
        sqlx::query(
            r#"
            INSERT INTO store_staff (id, store_id, email, login_id, phone, password_hash, role_id, status, display_name)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9)
            "#,
        )
        .bind(staff_id)
        .bind(store_uuid)
        .bind(email)
        .bind(login_id)
        .bind(phone)
        .bind(password_hash)
        .bind(role_uuid)
        .bind(status)
        .bind(display_name)
        .execute(self.db)
        .await
        .map_err(IdentityError::from)?;
        Ok(())
    }

    async fn role_key_by_id(
        &self,
        store_uuid: &uuid::Uuid,
        role_id: &str,
    ) -> IdentityResult<Option<String>> {
        let row = sqlx::query(
            r#"
            SELECT key
            FROM store_roles
            WHERE id = $1 AND store_id = $2
            "#,
        )
        .bind(role_id)
        .bind(store_uuid)
        .fetch_optional(self.db)
        .await
        .map_err(IdentityError::from)?;
        Ok(row.map(|row| row.get("key")))
    }

    async fn role_store_id(&self, role_id: &str) -> IdentityResult<Option<String>> {
        let row = sqlx::query("SELECT store_id::text as store_id FROM store_roles WHERE id = $1")
            .bind(role_id)
            .fetch_optional(self.db)
            .await
            .map_err(IdentityError::from)?;
        Ok(row.map(|row| row.get("store_id")))
    }

    async fn staff_summary(
        &self,
        store_uuid: &uuid::Uuid,
        staff_uuid: &uuid::Uuid,
    ) -> IdentityResult<Option<StaffSummaryRow>> {
        let row = sqlx::query(
            r#"
            SELECT ss.id::text as staff_id, ss.email, ss.login_id, ss.phone, ss.status,
                   ss.display_name, sr.id::text as role_id, sr.key as role_key
            FROM store_staff ss
            JOIN store_roles sr ON sr.id = ss.role_id
            WHERE ss.id = $1 AND ss.store_id = $2
            "#,
        )
        .bind(staff_uuid)
        .bind(store_uuid)
        .fetch_optional(self.db)
        .await
        .map_err(IdentityError::from)?;
        Ok(row.map(|row| StaffSummaryRow {
            staff_id: row.get("staff_id"),
            email: row.get("email"),
            login_id: row.get("login_id"),
            phone: row.get("phone"),
            status: row.get("status"),
            display_name: row.get("display_name"),
            role_id: row.get("role_id"),
            role_key: row.get("role_key"),
        }))
    }

    async fn store_name(&self, store_uuid: &uuid::Uuid) -> IdentityResult<Option<String>> {
        let row = sqlx::query("SELECT name FROM stores WHERE id = $1")
            .bind(store_uuid)
            .fetch_optional(self.db)
            .await
            .map_err(IdentityError::from)?;
        Ok(row.map(|row| row.get("name")))
    }

    async fn role_id_by_key(
        &self,
        store_uuid: &uuid::Uuid,
        role_key: &str,
    ) -> IdentityResult<Option<uuid::Uuid>> {
        let row = sqlx::query("SELECT id FROM store_roles WHERE store_id = $1 AND key = $2")
            .bind(store_uuid)
            .bind(role_key)
            .fetch_optional(self.db)
            .await
            .map_err(IdentityError::from)?;
        Ok(row.map(|row| row.get("id")))
    }

    async fn list_roles(&self, store_uuid: &uuid::Uuid) -> IdentityResult<Vec<RoleDetailRow>> {
        let rows = sqlx::query(
            r#"
            SELECT id::text as id, key, name, description
            FROM store_roles
            WHERE store_id = $1 AND key <> 'owner'
            ORDER BY created_at ASC
            "#,
        )
        .bind(store_uuid)
        .fetch_all(self.db)
        .await
        .map_err(IdentityError::from)?;

        Ok(rows
            .into_iter()
            .map(|row| RoleDetailRow {
                id: row.get("id"),
                key: row.get("key"),
                name: row.get("name"),
                description: row.get("description"),
            })
            .collect())
    }

    async fn list_role_permissions(
        &self,
        role_ids: &[uuid::Uuid],
    ) -> IdentityResult<Vec<(String, String)>> {
        let rows = sqlx::query(
            r#"
            SELECT srp.role_id::text as role_id, p.key as key
            FROM store_role_permissions srp
            JOIN permissions p ON p.id = srp.permission_id
            WHERE srp.role_id = ANY($1)
            "#,
        )
        .bind(role_ids)
        .fetch_all(self.db)
        .await
        .map_err(IdentityError::from)?;

        Ok(rows
            .into_iter()
            .map(|row| (row.get("role_id"), row.get("key")))
            .collect())
    }

    async fn permissions_by_keys(&self, keys: &[String]) -> IdentityResult<Vec<(String, String)>> {
        let rows = sqlx::query(
            r#"
            SELECT id::text as id, key
            FROM permissions
            WHERE key = ANY($1)
            "#,
        )
        .bind(keys)
        .fetch_all(self.db)
        .await
        .map_err(IdentityError::from)?;
        Ok(rows
            .into_iter()
            .map(|row| (row.get("id"), row.get("key")))
            .collect())
    }

    async fn role_attached(&self, role_uuid: &uuid::Uuid) -> IdentityResult<bool> {
        let row = sqlx::query(
            r#"
            SELECT 1
            FROM store_staff
            WHERE role_id = $1
            LIMIT 1
            "#,
        )
        .bind(role_uuid)
        .fetch_optional(self.db)
        .await
        .map_err(IdentityError::from)?;
        Ok(row.is_some())
    }

    async fn delete_role(
        &self,
        store_uuid: &uuid::Uuid,
        role_uuid: &uuid::Uuid,
    ) -> IdentityResult<Option<RoleRow>> {
        let row = sqlx::query(
            r#"
            DELETE FROM store_roles
            WHERE id = $1 AND store_id = $2
            RETURNING id::text as id, key, name
            "#,
        )
        .bind(role_uuid)
        .bind(store_uuid)
        .fetch_optional(self.db)
        .await
        .map_err(IdentityError::from)?;
        Ok(row.map(|row| RoleRow {
            id: row.get("id"),
            key: row.get("key"),
            name: row.get("name"),
        }))
    }

    async fn update_staff_role(
        &self,
        staff_uuid: &uuid::Uuid,
        store_uuid: &uuid::Uuid,
        role_uuid: &uuid::Uuid,
    ) -> IdentityResult<()> {
        sqlx::query(
            r#"
            UPDATE store_staff
            SET role_id = $1, updated_at = now()
            WHERE id = $2 AND store_id = $3
            "#,
        )
        .bind(role_uuid)
        .bind(staff_uuid)
        .bind(store_uuid)
        .execute(self.db)
        .await
        .map_err(IdentityError::from)?;
        Ok(())
    }

    async fn store_staff_exists_by_email(
        &self,
        store_uuid: &uuid::Uuid,
        email: &str,
    ) -> IdentityResult<bool> {
        let row = sqlx::query(
            r#"
            SELECT 1
            FROM store_staff
            WHERE store_id = $1 AND email = $2
            "#,
        )
        .bind(store_uuid)
        .bind(email)
        .fetch_optional(self.db)
        .await
        .map_err(IdentityError::from)?;
        Ok(row.is_some())
    }

    async fn invite_exists_by_email(
        &self,
        store_uuid: &uuid::Uuid,
        email: &str,
    ) -> IdentityResult<bool> {
        let row = sqlx::query(
            r#"
            SELECT 1
            FROM store_staff_invites
            WHERE store_id = $1 AND email = $2 AND accepted_at IS NULL
            "#,
        )
        .bind(store_uuid)
        .bind(email)
        .fetch_optional(self.db)
        .await
        .map_err(IdentityError::from)?;
        Ok(row.is_some())
    }

    async fn insert_staff_invite_tx<'e, E>(
        &self,
        exec: &mut E,
        staff_id: &uuid::Uuid,
        invite_id: &uuid::Uuid,
        store_uuid: &uuid::Uuid,
        email: &str,
        role_uuid: &uuid::Uuid,
        token: &str,
        created_by: Option<uuid::Uuid>,
        expires_at: chrono::DateTime<chrono::Utc>,
        display_name: Option<&str>,
    ) -> IdentityResult<()>
    where
        for<'c> &'c mut E: Executor<'c, Database = Postgres>,
    {
        sqlx::query(
            r#"
            INSERT INTO store_staff (id, store_id, email, role_id, status, display_name)
            VALUES ($1,$2,$3,$4,$5,$6)
            "#,
        )
        .bind(staff_id)
        .bind(store_uuid)
        .bind(email)
        .bind(role_uuid)
        .bind("invited")
        .bind(display_name)
        .execute(&mut *exec)
        .await
        .map_err(IdentityError::from)?;

        sqlx::query(
            r#"
            INSERT INTO store_staff_invites (id, store_id, email, role_id, token, created_by, expires_at)
            VALUES ($1,$2,$3,$4,$5,$6,$7)
            "#,
        )
        .bind(invite_id)
        .bind(store_uuid)
        .bind(email)
        .bind(role_uuid)
        .bind(token)
        .bind(created_by)
        .bind(expires_at)
        .execute(&mut *exec)
        .await
        .map_err(IdentityError::from)?;
        Ok(())
    }

    async fn current_owner_id(&self, store_uuid: &uuid::Uuid) -> IdentityResult<Option<String>> {
        let row = sqlx::query(
            r#"
            SELECT ss.id::text as staff_id
            FROM store_staff ss
            JOIN store_roles sr ON sr.id = ss.role_id
            WHERE ss.store_id = $1 AND sr.key = 'owner'
            "#,
        )
        .bind(store_uuid)
        .fetch_optional(self.db)
        .await
        .map_err(IdentityError::from)?;
        Ok(row.map(|row| row.get("staff_id")))
    }

    async fn staff_status(
        &self,
        store_uuid: &uuid::Uuid,
        staff_uuid: &uuid::Uuid,
    ) -> IdentityResult<Option<String>> {
        let row = sqlx::query(
            r#"
            SELECT ss.status
            FROM store_staff ss
            WHERE ss.id = $1 AND ss.store_id = $2
            "#,
        )
        .bind(staff_uuid)
        .bind(store_uuid)
        .fetch_optional(self.db)
        .await
        .map_err(IdentityError::from)?;
        Ok(row.map(|row| row.get("status")))
    }

    async fn fetch_active_staff_by_email(
        &self,
        store_uuid: &uuid::Uuid,
        email: &str,
    ) -> IdentityResult<Option<StaffAuthRow>> {
        let row = sqlx::query(
            r#"
            SELECT ss.id::text as id, ss.password_hash, sr.key as role_key
            FROM store_staff ss
            JOIN store_roles sr ON sr.id = ss.role_id
            WHERE ss.store_id = $1 AND ss.email = $2 AND ss.status = 'active'
            LIMIT 1
            "#,
        )
        .bind(store_uuid)
        .bind(email)
        .fetch_optional(self.db)
        .await
        .map_err(IdentityError::from)?;
        Ok(row.map(|row| StaffAuthRow {
            staff_id: row.get("id"),
            password_hash: row.get("password_hash"),
            role_key: row.get("role_key"),
        }))
    }

    async fn fetch_active_staff_by_login_id(
        &self,
        store_uuid: &uuid::Uuid,
        login_id: &str,
    ) -> IdentityResult<Option<StaffAuthRow>> {
        let row = sqlx::query(
            r#"
            SELECT ss.id::text as id, ss.password_hash, sr.key as role_key
            FROM store_staff ss
            JOIN store_roles sr ON sr.id = ss.role_id
            WHERE ss.store_id = $1 AND ss.login_id = $2 AND ss.status = 'active'
            LIMIT 1
            "#,
        )
        .bind(store_uuid)
        .bind(login_id)
        .fetch_optional(self.db)
        .await
        .map_err(IdentityError::from)?;
        Ok(row.map(|row| StaffAuthRow {
            staff_id: row.get("id"),
            password_hash: row.get("password_hash"),
            role_key: row.get("role_key"),
        }))
    }

    async fn fetch_active_staff_by_phone(
        &self,
        store_uuid: &uuid::Uuid,
        phone: &str,
    ) -> IdentityResult<Option<StaffAuthRow>> {
        let row = sqlx::query(
            r#"
            SELECT ss.id::text as id, ss.password_hash, sr.key as role_key
            FROM store_staff ss
            JOIN store_roles sr ON sr.id = ss.role_id
            WHERE ss.store_id = $1 AND ss.phone = $2 AND ss.status = 'active'
            LIMIT 1
            "#,
        )
        .bind(store_uuid)
        .bind(phone)
        .fetch_optional(self.db)
        .await
        .map_err(IdentityError::from)?;
        Ok(row.map(|row| StaffAuthRow {
            staff_id: row.get("id"),
            password_hash: row.get("password_hash"),
            role_key: row.get("role_key"),
        }))
    }
}

impl<'a> PgIdentityRepository<'a> {
    pub fn new(db: &'a sqlx::PgPool) -> Self {
        Self { db }
    }

    pub async fn insert_role_tx<'e, E>(
        &self,
        exec: E,
        store_uuid: &uuid::Uuid,
        role_uuid: &uuid::Uuid,
        key: &str,
        name: &str,
        description: &str,
    ) -> IdentityResult<RoleRow>
    where
        E: Executor<'e, Database = Postgres>,
    {
        sqlx::query(
            r#"
            INSERT INTO store_roles (id, store_id, key, name, description)
            VALUES ($1,$2,$3,$4,$5)
            "#,
        )
        .bind(role_uuid)
        .bind(store_uuid)
        .bind(key)
        .bind(name)
        .bind(description)
        .execute(exec)
        .await
        .map_err(IdentityError::from)?;

        Ok(RoleRow {
            id: role_uuid.to_string(),
            key: key.to_string(),
            name: Some(name.to_string()),
        })
    }

    pub async fn update_role_tx<'e, E>(
        &self,
        exec: E,
        store_uuid: &uuid::Uuid,
        role_uuid: &uuid::Uuid,
        name: &str,
        description: &str,
    ) -> IdentityResult<Option<RoleDetailRow>>
    where
        E: Executor<'e, Database = Postgres>,
    {
        let row = sqlx::query(
            r#"
            UPDATE store_roles
            SET name = COALESCE(NULLIF($1, ''), name),
                description = COALESCE(NULLIF($2, ''), description),
                updated_at = now()
            WHERE id = $3 AND store_id = $4
            RETURNING id::text as id, key, name, description
            "#,
        )
        .bind(name)
        .bind(description)
        .bind(role_uuid)
        .bind(store_uuid)
        .fetch_optional(exec)
        .await
        .map_err(IdentityError::from)?;

        Ok(row.map(|row| RoleDetailRow {
            id: row.get("id"),
            key: row.get("key"),
            name: row.get("name"),
            description: row.get("description"),
        }))
    }

    pub async fn delete_role_permissions_tx<'e, E>(
        &self,
        exec: E,
        role_uuid: &uuid::Uuid,
    ) -> IdentityResult<()>
    where
        E: Executor<'e, Database = Postgres>,
    {
        sqlx::query("DELETE FROM store_role_permissions WHERE role_id = $1")
            .bind(role_uuid)
            .execute(exec)
            .await
            .map_err(IdentityError::from)?;
        Ok(())
    }

    pub async fn insert_role_permission_tx<'e, E>(
        &self,
        exec: E,
        role_uuid: &uuid::Uuid,
        permission_uuid: &uuid::Uuid,
    ) -> IdentityResult<()>
    where
        E: Executor<'e, Database = Postgres>,
    {
        sqlx::query(
            r#"
            INSERT INTO store_role_permissions (role_id, permission_id)
            VALUES ($1,$2)
            ON CONFLICT DO NOTHING
            "#,
        )
        .bind(role_uuid)
        .bind(permission_uuid)
        .execute(exec)
        .await
        .map_err(IdentityError::from)?;
        Ok(())
    }

    pub async fn insert_staff_tx<'e, E>(
        &self,
        exec: E,
        staff_id: &uuid::Uuid,
        store_uuid: &uuid::Uuid,
        email: Option<&str>,
        login_id: Option<&str>,
        phone: Option<&str>,
        password_hash: &str,
        role_uuid: &uuid::Uuid,
        status: &str,
        display_name: Option<&str>,
    ) -> IdentityResult<()>
    where
        E: Executor<'e, Database = Postgres>,
    {
        sqlx::query(
            r#"
            INSERT INTO store_staff (id, store_id, email, login_id, phone, password_hash, role_id, status, display_name)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9)
            "#,
        )
        .bind(staff_id)
        .bind(store_uuid)
        .bind(email)
        .bind(login_id)
        .bind(phone)
        .bind(password_hash)
        .bind(role_uuid)
        .bind(status)
        .bind(display_name)
        .execute(exec)
        .await
        .map_err(IdentityError::from)?;
        Ok(())
    }

    pub async fn update_staff_tx<'e, E>(
        &self,
        exec: E,
        staff_uuid: &uuid::Uuid,
        store_uuid: &uuid::Uuid,
        role_id: &str,
        status: &str,
        display_name: &str,
    ) -> IdentityResult<Option<StaffSummaryRow>>
    where
        E: Executor<'e, Database = Postgres>,
    {
        let row = sqlx::query(
            r#"
            UPDATE store_staff
            SET role_id = COALESCE(NULLIF($1, '')::uuid, role_id),
                status = COALESCE(NULLIF($2, ''), status),
                display_name = COALESCE(NULLIF($3, ''), display_name),
                updated_at = now()
            WHERE id = $4 AND store_id = $5
            RETURNING id::text as staff_id, email, login_id, phone, status, role_id::text as role_id, display_name
            "#,
        )
        .bind(role_id)
        .bind(status)
        .bind(display_name)
        .bind(staff_uuid)
        .bind(store_uuid)
        .fetch_optional(exec)
        .await
        .map_err(IdentityError::from)?;

        Ok(row.map(|row| StaffSummaryRow {
            staff_id: row.get("staff_id"),
            email: row.get("email"),
            login_id: row.get("login_id"),
            phone: row.get("phone"),
            status: row.get("status"),
            display_name: row.get("display_name"),
            role_id: row.get("role_id"),
            role_key: String::new(),
        }))
    }

    pub async fn update_staff_role_tx<'e, E>(
        &self,
        exec: E,
        staff_uuid: &uuid::Uuid,
        store_uuid: &uuid::Uuid,
        role_uuid: &uuid::Uuid,
    ) -> IdentityResult<()>
    where
        E: Executor<'e, Database = Postgres>,
    {
        sqlx::query(
            r#"
            UPDATE store_staff
            SET role_id = $1, updated_at = now()
            WHERE id = $2 AND store_id = $3
            "#,
        )
        .bind(role_uuid)
        .bind(staff_uuid)
        .bind(store_uuid)
        .execute(exec)
        .await
        .map_err(IdentityError::from)?;
        Ok(())
    }

    pub async fn delete_role_tx<'e, E>(
        &self,
        exec: E,
        store_uuid: &uuid::Uuid,
        role_uuid: &uuid::Uuid,
    ) -> IdentityResult<Option<RoleRow>>
    where
        E: Executor<'e, Database = Postgres>,
    {
        let row = sqlx::query(
            r#"
            DELETE FROM store_roles
            WHERE id = $1 AND store_id = $2
            RETURNING id::text as id, key, name
            "#,
        )
        .bind(role_uuid)
        .bind(store_uuid)
        .fetch_optional(exec)
        .await
        .map_err(IdentityError::from)?;
        Ok(row.map(|row| RoleRow {
            id: row.get("id"),
            key: row.get("key"),
            name: row.get("name"),
        }))
    }
}
