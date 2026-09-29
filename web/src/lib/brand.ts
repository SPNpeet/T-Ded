// ตั้งค่าแบรนด์ตอน build ด้วย VITE_BRAND (teedet = ค่าเริ่มต้น, nb = หนองบัว ฟีด มิลล์)
export type Brand = {
  id: 'teedet' | 'nb'
  appName: string
  shortName: string
  tagline: string
  companyTh: string
  /** กรองแคตตาล็อกเฉพาะกลุ่มสินค้านี้ (null = ทุกยี่ห้อ) */
  productGroup: string | null
  /** หน้าแรกเริ่มที่ตัววางโปรแกรมการเลี้ยง */
  plannerFirst: boolean
  theme: Record<string, string>
}

const BRANDS: Record<string, Brand> = {
  teedet: {
    id: 'teedet',
    appName: 'ทีเด็ดปลาน้ำจืด',
    shortName: 'ทีเด็ดปลา',
    tagline: 'ด้วยอาหารคุณภาพ และคำปรึกษาจากมืออาชีพ',
    companyTh: 'ทีเด็ดปลาน้ำจืด',
    productGroup: null,
    plannerFirst: false,
    theme: {},
  },
  nb: {
    id: 'nb',
    appName: 'NB โปรแกรมเลี้ยงปลา',
    shortName: 'NB เลี้ยงปลา',
    tagline: 'รู้ล่วงหน้าว่าปลาโตเท่าไร ด้วยอาหารเบอร์ไหน',
    companyTh: 'บริษัท หนองบัว ฟีด มิลล์ จำกัด',
    productGroup: 'nbdc',
    plannerFirst: true,
    // สีชั่วคราวจากกระสอบ Pro Plus (น้ำเงิน/ส้ม) รอ CI จริงของลูกค้า
    theme: { '--navy': '#123B7A', '--navy-2': '#1F4FA3', '--cyan': '#F28C28', '--cyan-deep': '#C8680F', '--cyan-tint': '#FDEEDD' },
  },
}

export const brand: Brand = BRANDS[(import.meta.env.VITE_BRAND as string) || 'teedet'] ?? BRANDS.teedet

export function applyBrandTheme() {
  const root = document.documentElement
  for (const [k, v] of Object.entries(brand.theme)) root.style.setProperty(k, v)
  document.title = brand.appName
}
