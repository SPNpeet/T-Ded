// แพตช์ server ให้รู้จักตาราง feed_products (รันครั้งเดียวจาก web/)
import { readFileSync, writeFileSync } from 'node:fs'
const R = (p) => '../crates/server/' + p
const edit = (file, pairs) => {
  let c = readFileSync(R(file), 'utf8')
  for (const [a, b] of pairs) {
    if (!c.includes(a)) throw new Error(file + ' missing: ' + a.slice(0, 60))
    c = c.replace(a, b)
  }
  writeFileSync(R(file), c)
}
edit('src/db.rs', [[`("0002_feed_nutrition", include_str!("../migrations/0002_feed_nutrition.sql")),`, `("0002_feed_nutrition", include_str!("../migrations/0002_feed_nutrition.sql")),\n    ("0003_feed_products", include_str!("../migrations/0003_feed_products.sql")),`]])
edit('src/main.rs', [
  ['mod line;', 'mod line;\nmod products;'],
  ['    line::spawn_scheduler(state.clone());', '    products::seed_if_empty(&state).await.expect("seed feed products");\n    line::spawn_scheduler(state.clone());'],
  ['        .route("/nutrition-ingredients", get(calc::nutrition_ingredients))', '        .route("/nutrition-ingredients", get(calc::nutrition_ingredients))\n        .route("/feed-products", get(products::list).post(products::create))\n        .route("/feed-products/{id}", axum::routing::patch(products::update).delete(products::remove))'],
])
console.log('server patched')
