// สร้างโลโก้และไอคอนแอป "ทีเด็ดปลาน้ำจืด" ตามอัตลักษณ์เดิม: ตัวอักษรกรมท่า เงาฟ้า/ชมพู สโลแกนในเครื่องหมายคำพูด
// รัน: node scripts/brand.mjs  (ผลลัพธ์: public/icons/*, public/logo.svg, brand/*)
import sharp from 'sharp'
import { mkdirSync, writeFileSync } from 'node:fs'
import { resolve, dirname } from 'node:path'
import { fileURLToPath } from 'node:url'

const here = dirname(fileURLToPath(import.meta.url))
const root = resolve(here, '..')
const fontsDir = resolve(root, 'public/fonts').replace(/\\/g, '/')
mkdirSync(resolve(root, 'brand'), { recursive: true })
mkdirSync(resolve(root, 'public/icons'), { recursive: true })
const fcPath = resolve(root, 'brand/fonts.conf')
writeFileSync(
  fcPath,
  `<?xml version="1.0"?><!DOCTYPE fontconfig SYSTEM "fonts.dtd"><fontconfig><dir>${fontsDir}</dir><dir>C:/Windows/Fonts</dir><cachedir>${resolve(root, 'brand/.fc-cache').replace(/\\/g, '/')}</cachedir></fontconfig>`,
)
process.env.FONTCONFIG_FILE = fcPath

const NAVY = '#1B2235'
const CYAN = '#2ED3F0'
const PINK = '#E23DA6'
const MUTED = '#3B4556'

// ตัวอักษรแบบเงาสองสี (ฟ้าซ้ายบน ชมพูขวาล่าง) เหมือนโลโก้เดิม
function glitchText(txt, x, y, size, anchor = 'middle', fill = NAVY) {
  const off = size * 0.028
  return `<text x="${x + off}" y="${y + off}" font-family="Prompt" font-weight="800" font-size="${size}" fill="${PINK}" text-anchor="${anchor}">${txt}</text>
  <text x="${x - off}" y="${y - off}" font-family="Prompt" font-weight="800" font-size="${size}" fill="${CYAN}" text-anchor="${anchor}">${txt}</text>
  <text x="${x}" y="${y}" font-family="Prompt" font-weight="800" font-size="${size}" fill="${fill}" text-anchor="${anchor}">${txt}</text>`
}

// โลโก้หลัก แนวตั้ง 2 บรรทัด + สโลแกน
function wordmark({ w = 1400, h = 1150, bg = '#FFFFFF', dark = false, tagline = true }) {
  const fill = dark ? '#FFFFFF' : NAVY
  const size = w * 0.19
  const cx = w / 2
  const y1 = h * 0.42
  const y2 = h * 0.7
  const tag = tagline
    ? `<text x="${cx}" y="${h * 0.86}" font-family="Sarabun" font-weight="700" font-size="${w * 0.045}" fill="${dark ? '#DDE7F0' : MUTED}" text-anchor="middle">"ด้วยอาหารคุณภาพ &amp; คำปรึกษาจากมืออาชีพ"</text>`
    : ''
  return `<svg xmlns="http://www.w3.org/2000/svg" width="${w}" height="${h}" viewBox="0 0 ${w} ${h}">
  ${bg === 'none' ? '' : `<rect width="${w}" height="${h}" fill="${bg}"/>`}
  ${glitchText('ทีเด็ด', cx, y1, size, 'middle', fill)}
  ${glitchText('ปลาน้ำจืด', cx, y2, size, 'middle', fill)}
  ${tag}
</svg>`
}

// โลโก้แนวนอน บรรทัดเดียว
function horizontal({ w = 2200, h = 520, bg = '#FFFFFF', dark = false, tagline = true }) {
  const fill = dark ? '#FFFFFF' : NAVY
  const size = h * 0.5
  const cx = w / 2
  const tag = tagline
    ? `<text x="${cx}" y="${h * 0.87}" font-family="Sarabun" font-weight="700" font-size="${h * 0.11}" fill="${dark ? '#DDE7F0' : MUTED}" text-anchor="middle">"ด้วยอาหารคุณภาพ &amp; คำปรึกษาจากมืออาชีพ"</text>`
    : ''
  return `<svg xmlns="http://www.w3.org/2000/svg" width="${w}" height="${h}" viewBox="0 0 ${w} ${h}">
  ${bg === 'none' ? '' : `<rect width="${w}" height="${h}" fill="${bg}"/>`}
  ${glitchText('ทีเด็ดปลาน้ำจืด', cx, h * 0.56, size, 'middle', fill)}
  ${tag}
</svg>`
}

// ไอคอนแอป (สี่เหลี่ยมมน): พื้นขาว ตัวอักษร 2 บรรทัดแบบเดียวกับโลโก้ ให้จำได้ทันที
function appIcon({ size = 1024, radius = 0, bg = '#FFFFFF', dark = false }) {
  const fill = dark ? '#FFFFFF' : NAVY
  const fs = size * 0.27
  return `<svg xmlns="http://www.w3.org/2000/svg" width="${size}" height="${size}" viewBox="0 0 ${size} ${size}">
  <rect width="${size}" height="${size}" rx="${radius}" fill="${bg}"/>
  ${glitchText('ทีเด็ด', size / 2, size * 0.45, fs, 'middle', fill)}
  ${glitchText('ปลาน้ำจืด', size / 2, size * 0.76, fs * 0.72, 'middle', fill)}
</svg>`
}

// เครื่องหมายเล็กสำหรับแถบบนสุดของแอป (คำเดียว)
function compactMark({ size = 256, dark = false }) {
  const fill = dark ? '#FFFFFF' : NAVY
  return `<svg xmlns="http://www.w3.org/2000/svg" width="${size}" height="${size}" viewBox="0 0 ${size} ${size}">
  <rect width="${size}" height="${size}" rx="${size * 0.22}" fill="${dark ? NAVY : '#FFFFFF'}"/>
  ${glitchText('ทีเด็ด', size / 2, size * 0.66, size * 0.42, 'middle', fill)}
</svg>`
}

async function png(svg, out, opts = {}) {
  let img = sharp(Buffer.from(svg), { density: 144 })
  if (opts.resize) img = img.resize(opts.resize, opts.resize)
  await img.png().toFile(out)
}

// ---- ไอคอนแอป ----
const iconSvg = appIcon({ size: 1024 })
writeFileSync(resolve(root, 'public/logo.svg'), compactMark({ size: 256, dark: true }))
writeFileSync(resolve(root, 'brand/app-icon.svg'), iconSvg)
for (const s of [16, 32, 48, 72, 96, 128, 144, 152, 180, 192, 256, 384, 512, 1024]) {
  await png(iconSvg, resolve(root, `public/icons/icon-${s}.png`), { resize: s })
}
await png(iconSvg, resolve(root, 'public/icons/apple-touch-icon.png'), { resize: 180 })
// maskable: safe zone 20% ทำตัวอักษรเล็กลง
await png(appIcon({ size: 1024 }).replace(/font-size="([\d.]+)"/g, (m, v) => `font-size="${v * 0.8}"`), resolve(root, 'public/icons/icon-512-maskable.png'), { resize: 512 })
await png(iconSvg, resolve(root, 'brand/app-icon-1024.png'))
await png(compactMark({ size: 512, dark: true }), resolve(root, 'brand/mark-compact-dark.png'))
await png(compactMark({ size: 512 }), resolve(root, 'brand/mark-compact-light.png'))

// ---- โลโก้สำหรับเพจ/สื่อ ----
const outputs = [
  ['logo-stacked-white', wordmark({})],
  ['logo-stacked-transparent', wordmark({ bg: 'none' })],
  ['logo-stacked-dark', wordmark({ bg: NAVY, dark: true })],
  ['logo-stacked-notagline', wordmark({ tagline: false })],
  ['logo-horizontal-white', horizontal({})],
  ['logo-horizontal-transparent', horizontal({ bg: 'none' })],
  ['logo-horizontal-dark', horizontal({ bg: NAVY, dark: true })],
  ['facebook-cover', horizontal({ w: 1640, h: 924 })],
  ['profile-square', appIcon({ size: 1080 })],
  ['profile-square-dark', appIcon({ size: 1080, bg: NAVY, dark: true })],
]
for (const [name, svg] of outputs) {
  writeFileSync(resolve(root, `brand/${name}.svg`), svg)
  await png(svg, resolve(root, `brand/${name}.png`))
}
console.log('brand assets written')
