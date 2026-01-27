<script setup>
import TitleBar from "../components/TitleBar.vue";
import { useDialog } from "../composables/useDialog";

const { alert } = useDialog();

const appInfo = {
  name: "Mi心动",
  version: "0.1.0",
  description: "一个跨平台的蓝牙心率监测应用，实时显示心率数据",
  author: "maybug",
  authorGithub: "https://github.com/may-bug",
  repository: "https://github.com/may-bug/MiHeart",
  license: "MIT",
  updateDate: "2026-01-27",
  releaseDate: "2026-01-20",
  technologies: [
    { name: "Vue 3", version: "3.5.2"},
    { name: "Tauri", version: "2.9.5"},
    { name: "Rust", version: "1.95"},
  ],
  features: [
    "✓ 实时蓝牙心率数据采集",
    "✓ 浮窗显示，不打扰工作",
    "✓ 跨平台支持（Windows/Mac/Linux）",
    "✓ 开源并免费"
  ]
};

const openLink = async (url) => {
  try {
    // await open(url);
  } catch (error) {
    useDialog({ type: "error", message: "无法打开链接，请手动访问", title: "提示" });
  }
};
</script>

<template>
  <div class="about-container">
    <TitleBar title="关于" />
    
    <div class="about-content">
      <!-- 应用信息卡片 -->
      <div class="about-card">
        <div class="app-header">
          <div class="app-icon">❤️</div>
          <div class="app-info">
            <h1 class="app-name">{{ appInfo.name }}</h1>
            <p class="app-version">v{{ appInfo.version }}</p>
            <p class="app-desc">{{ appInfo.description }}</p>
          </div>
        </div>
      </div>

      <!-- 版本信息 -->
      <div class="about-card">
        <h3 class="card-title">版本信息</h3>
        <div class="info-list">
          <div class="info-item">
            <span class="info-label">当前版本</span>
            <span class="info-value">v{{ appInfo.version }}</span>
          </div>
          <div class="info-item">
            <span class="info-label">发布日期</span>
            <span class="info-value">{{ appInfo.releaseDate }}</span>
          </div>
          <div class="info-item">
            <span class="info-label">更新日期</span>
            <span class="info-value">{{ appInfo.updateDate }}</span>
          </div>
          <div class="info-item">
            <span class="info-label">开源许可证</span>
            <span class="info-value license">{{ appInfo.license }}</span>
          </div>
        </div>
      </div>

      <!-- 作者信息 -->
      <div class="about-card">
        <h3 class="card-title">作者信息</h3>
        <div class="author-section">
          <div class="author-name">{{ appInfo.author }}</div>
          <div class="author-links">
            <button 
              @click="openLink(appInfo.authorGithub)" 
              class="link-btn github-btn"
            >
              <span>GitHub 主页</span>
            </button>
            <button 
              @click="openLink(appInfo.repository)" 
              class="link-btn repo-btn"
            >
              <span>项目仓库</span>
            </button>
          </div>
        </div>
      </div>

      <!-- 技术栈 -->
      <div class="about-card">
        <h3 class="card-title">技术栈</h3>
        <div class="tech-grid">
          <div v-for="tech in appInfo.technologies" :key="tech.name" class="tech-item">
            <div class="tech-info">
              <div class="tech-name">{{ tech.name }}</div>
              <div class="tech-version">v{{ tech.version }}</div>
            </div>
          </div>
        </div>
      </div>

      <!-- 主要特性 -->
      <div class="about-card">
        <h3 class="card-title">主要特性</h3>
        <div class="features-list">
          <div v-for="(feature, index) in appInfo.features" :key="index" class="feature-item">
            {{ feature }}
          </div>
        </div>
      </div>

      <!-- 许可证信息 -->
      <div class="about-card license-card">
        <h3 class="card-title">许可证</h3>
        <p class="license-text">
          本项目采用 <strong>{{ appInfo.license }} License</strong> 开源许可证发布。
          这意味着您可以自由地使用、修改和分发本软件，但需要保留原始版权声明。
        </p>
        <a 
          href="https://opensource.org/licenses/MIT" 
          target="_blank" 
          class="license-link"
        >
          查看完整许可证文本 →
        </a>
      </div>

      <!-- 底部信息 -->
      <div class="footer-section">
        <p class="footer-text">感谢使用 {{ appInfo.name }}！</p>
        <p class="footer-subtext">如有问题或建议，欢迎在 GitHub 上提交 Issue</p>
      </div>
    </div>
  </div>
</template>

<style scoped>
* {
  box-sizing: border-box;
  margin: 0;
  padding: 0;
}

.about-container {
  width: 100%;
  height: 100vh;
  background: linear-gradient(135deg, #f5f7fa 0%, #c3cfe2 100%);
  display: flex;
  flex-direction: column;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
  color: #333;
}

.about-content {
  flex: 1;
  overflow-y: auto;
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

/* 自定义滚动条 */
.about-content::-webkit-scrollbar {
  width: 8px;
}

.about-content::-webkit-scrollbar-track {
  background: rgba(255, 255, 255, 0.3);
  border-radius: 10px;
}

.about-content::-webkit-scrollbar-thumb {
  background: linear-gradient(180deg, rgba(76, 175, 80, 0.6) 0%, rgba(66, 133, 244, 0.6) 100%);
  border-radius: 10px;
  transition: all 0.3s ease;
}

.about-content::-webkit-scrollbar-thumb:hover {
  background: linear-gradient(180deg, rgba(76, 175, 80, 0.8) 0%, rgba(66, 133, 244, 0.8) 100%);
}

/* 卡片样式 */
.about-card {
  background: rgba(255, 255, 255, 0.7);
  backdrop-filter: blur(10px);
  border: 1px solid rgba(255, 255, 255, 0.5);
  border-radius: 12px;
  padding: 16px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.05);
  transition: all 0.3s ease;
}

.about-card:hover {
  background: rgba(255, 255, 255, 0.8);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.08);
}

.card-title {
  font-size: 16px;
  font-weight: 700;
  margin-bottom: 12px;
  color: #333;
  display: flex;
  align-items: center;
  gap: 8px;
}

/* App Header */
.app-header {
  display: flex;
  align-items: center;
  gap: 16px;
}

.app-icon {
  font-size: 48px;
  flex-shrink: 0;
}

.app-info {
  flex: 1;
}

.app-name {
  font-size: 24px;
  font-weight: 800;
  color: #333;
  margin: 0 0 4px 0;
}

.app-version {
  font-size: 12px;
  color: #666;
  margin: 0 0 8px 0;
}

.app-desc {
  font-size: 13px;
  color: #555;
  margin: 0;
  line-height: 1.5;
}

/* 信息列表 */
.info-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.info-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 0;
  border-bottom: 1px solid rgba(0, 0, 0, 0.05);
}

.info-item:last-child {
  border-bottom: none;
}

.info-label {
  font-size: 13px;
  color: #666;
  font-weight: 500;
}

.info-value {
  font-size: 13px;
  color: #333;
  font-weight: 600;
}

.info-value.license {
  background: linear-gradient(135deg, #ff9800 0%, #ff6f00 100%);
  color: white;
  padding: 2px 8px;
  border-radius: 4px;
  font-size: 12px;
}

/* 作者信息 */
.author-section {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.author-name {
  font-size: 16px;
  font-weight: 700;
  color: #333;
}

.author-links {
  display: flex;
  gap: 10px;
  flex-wrap: wrap;
}

.link-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 12px;
  border: none;
  border-radius: 6px;
  font-size: 12px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.3s ease;
}

.github-btn {
  background: rgba(0, 0, 0, 0.05);
  color: #333;
}

.github-btn:hover {
  background: rgba(0, 0, 0, 0.1);
  transform: translateY(-2px);
  box-shadow: 0 4px 8px rgba(0, 0, 0, 0.1);
}

.repo-btn {
  background: linear-gradient(135deg, #4285f4 0%, #1a73e8 100%);
  color: white;
}

.repo-btn:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(66, 133, 244, 0.4);
}

/* 技术栈 */
.tech-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(100px, 1fr));
  gap: 10px;
}

.tech-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  padding: 12px;
  background: rgba(0, 0, 0, 0.02);
  border: 1px solid rgba(0, 0, 0, 0.05);
  border-radius: 8px;
  transition: all 0.3s ease;
}

.tech-item:hover {
  background: rgba(66, 133, 244, 0.1);
  border-color: rgba(66, 133, 244, 0.3);
  transform: translateY(-2px);
}

.tech-icon {
  font-size: 24px;
}

.tech-info {
  text-align: center;
}

.tech-name {
  font-size: 12px;
  font-weight: 600;
  color: #333;
}

.tech-version {
  font-size: 11px;
  color: #999;
}

/* 特性列表 */
.features-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.feature-item {
  font-size: 13px;
  color: #555;
  padding: 6px 0;
  padding-left: 8px;
  border-left: 3px solid #4caf50;
}

/* 许可证卡片 */
.license-card {
  border: 1px solid rgba(255, 152, 0, 0.3);
}

.license-text {
  font-size: 13px;
  color: #555;
  line-height: 1.6;
  margin-bottom: 10px;
}

.license-link {
  display: inline-block;
  color: #4285f4;
  text-decoration: none;
  font-size: 12px;
  font-weight: 600;
  transition: all 0.3s ease;
}

.license-link:hover {
  color: #1a73e8;
  text-decoration: underline;
}

/* 底部信息 */
.footer-section {
  text-align: center;
  padding: 20px 16px;
  background: rgba(255, 255, 255, 0.5);
  border-top: 1px solid rgba(0, 0, 0, 0.05);
  margin-top: 4px;
}

.footer-text {
  font-size: 14px;
  font-weight: 600;
  color: #333;
  margin: 0 0 6px 0;
}

.footer-subtext {
  font-size: 12px;
  color: #999;
  margin: 0;
}

@media (max-width: 600px) {
  .tech-grid {
    grid-template-columns: repeat(auto-fit, minmax(80px, 1fr));
  }

  .author-links {
    flex-direction: column;
  }

  .link-btn {
    width: 100%;
    justify-content: center;
  }

  .app-header {
    flex-direction: column;
    text-align: center;
  }

  .app-icon {
    font-size: 40px;
  }

  .app-name {
    font-size: 20px;
  }
}
</style>