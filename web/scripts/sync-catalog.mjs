// สร้างแคตตาล็อกอาหารในแอป (ใช้ตอนไม่มีเซิร์ฟเวอร์) จากไฟล์ seed ฝั่งเซิร์ฟเวอร์ ให้ข้อมูลมาจากที่เดียว
import { readFileSync, writeFileSync } from 'node:fs'
const seed = JSON.parse(readFileSync(new URL('../../crates/server/seed/feed_products.json', import.meta.url), 'utf8'))
const all = seed.map((p, i) => ({ id: 'static-' + i, active: 1, ...p }))
writeFileSync(new URL('../public/feed-products.json', import.meta.url), JSON.stringify(all))
writeFileSync(new URL('../public-nb/feed-products.json', import.meta.url), JSON.stringify(all.filter((p) => p.brand_group === 'nbdc')))
console.log('catalog synced:', all.length, 'all /', all.filter((p) => p.brand_group === 'nbdc').length, 'nbdc')
