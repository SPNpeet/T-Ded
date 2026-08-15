// ปุ่มต้องกดได้เสมอ ถ้ากรอกไม่ครบให้บอกว่าขาดอะไร (ปุ่มจาง ๆ กดไม่ได้ทำให้ผู้ใช้คิดว่าแอปพัง)
import { readFileSync, writeFileSync } from 'node:fs'
const edit = (file, pairs) => {
  let c = readFileSync(file, 'utf8')
  for (const [a, b] of pairs) {
    if (!c.includes(a)) throw new Error(file + ' missing: ' + a.slice(0, 60))
    c = c.split(a).join(b)
  }
  writeFileSync(file, c)
}

edit('src/pages/NewCrop.svelte', [
  ['  async function save() {\n    busy = true', "  async function save() {\n    const miss: string[] = []\n    if (!count || !(parseInt(count) > 0)) miss.push('จำนวนที่ปล่อย')\n    if (!weight || !(parseFloat(weight) > 0)) miss.push('น้ำหนักเฉลี่ยตอนปล่อย')\n    if (miss.length) return toast('กรอก' + miss.join(' และ ') + 'ก่อนครับ', 'error', 3500)\n    busy = true"],
  ['disabled={busy || !count || !weight}', 'disabled={busy}'],
])
edit('src/pages/Diseases.svelte', [
  ['  async function save() {\n    busy = true', "  async function save() {\n    if (!symptom) return toast('เลือกอาการที่พบก่อนครับ', 'error')\n    busy = true"],
  ['disabled={busy || !symptom}', 'disabled={busy}'],
])
edit('src/pages/Ponds.svelte', [
  ['  async function addPond() {\n    busy = true', "  async function addPond() {\n    if (!name.trim()) return toast('ตั้งชื่อบ่อก่อนครับ เช่น บ่อ 1', 'error')\n    busy = true"],
  ['disabled={busy || !name}', 'disabled={busy}'],
])
edit('src/pages/Prices.svelte', [
  ['  async function save() {\n    busy = true', "  async function save() {\n    if (!price || !(parseFloat(price) > 0)) return toast('กรอกราคาบาท/กก. ก่อนครับ', 'error')\n    busy = true"],
  ['disabled={busy || !price}', 'disabled={busy}'],
])
edit('src/pages/Admin.svelte', [
  ['  async function createUser() {\n    busy = true', "  async function createUser() {\n    const miss: string[] = []\n    if (!newUser.name) miss.push('ชื่อ')\n    if (!newUser.phone) miss.push('เบอร์โทร')\n    if (!newUser.pin) miss.push('PIN')\n    if (miss.length) return toast('กรอก' + miss.join(', ') + 'ก่อน', 'error')\n    busy = true"],
  ['disabled={busy || !newUser.name || !newUser.phone || !newUser.pin}', 'disabled={busy}'],
  ['  async function sendAnn() {\n    busy = true', "  async function sendAnn() {\n    if (!ann.title.trim()) return toast('ใส่หัวข้อประกาศก่อน', 'error')\n    busy = true"],
  ['disabled={busy || !ann.title}', 'disabled={busy}'],
])
edit('src/pages/Settings.svelte', [
  ['  async function changePin() {\n    try {', "  async function changePin() {\n    if (!oldPin || !newPin) return toast('กรอก PIN เดิมและ PIN ใหม่ก่อน', 'error')\n    try {"],
  ['disabled={!oldPin || !newPin}', ''],
  ['  async function addWorker() {\n    try {', "  async function addWorker() {\n    if (!workerName || !workerPhone || !workerPin) return toast('กรอกชื่อ เบอร์โทร และ PIN ให้ครบ', 'error')\n    try {"],
  ['disabled={!workerName || !workerPhone || !workerPin}', ''],
  ['  async function addFarm() {\n    try {', "  async function addFarm() {\n    if (!newFarmName.trim()) return toast('ตั้งชื่อฟาร์มใหม่ก่อน', 'error')\n    try {"],
  ['disabled={!newFarmName}', ''],
])
console.log('buttons always clickable')
