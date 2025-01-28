use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use imgfloat::domain::db::SqliteDbService;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

pub struct TestDbService(pub SqliteDbService);

impl TestDbService {
    pub fn new() -> Self {
        let service = SqliteDbService::new(":memory:").unwrap();
        let mut conn = service.pool.get().unwrap();
        conn.run_pending_migrations(MIGRATIONS).unwrap();
        Self(service)
    }
}
