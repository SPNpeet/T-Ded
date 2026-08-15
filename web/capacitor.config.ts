import type { CapacitorConfig } from '@capacitor/cli'

const config: CapacitorConfig = {
  appId: 'com.teedetpla.app',
  appName: 'ทีเด็ดปลาน้ำจืด',
  webDir: 'dist',
  android: {
    allowMixedContent: false,
  },
  ios: {
    contentInset: 'always',
  },
  server: {
    androidScheme: 'https',
    // แอปโหลดไฟล์จากในเครื่อง ส่วน API ผู้ใช้ตั้งที่อยู่เองในหน้า "ตั้งค่าเซิร์ฟเวอร์"
    cleartext: false,
  },
  plugins: {
    SplashScreen: {
      launchShowDuration: 800,
      backgroundColor: '#1B2440',
      showSpinner: false,
    },
  },
}

export default config
