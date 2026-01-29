<template>
  <div class="splash-screen-wrapper" :class="{ 'fade-out': isFading }">
    <div class="splash-window">
      <div class="top-info">TAURI BOOTSTRAP</div>

      <div class="content-area">
        <div class="heart-pulse-container">
          <svg viewBox="0 0 100 40" preserveAspectRatio="xMidYMid meet">
            <defs>
              <linearGradient id="heartGradient" x1="0%" y1="0%" x2="100%" y2="0%">
                <stop offset="0%" stop-color="#ff4d4d" stop-opacity="0.1" />
                <stop offset="20%" stop-color="#ff4d4d" stop-opacity="0.8" />
                <stop offset="50%" stop-color="#ff4d4d" stop-opacity="1" />
                <stop offset="80%" stop-color="#ff4d4d" stop-opacity="0.8" />
                <stop offset="100%" stop-color="#ff4d4d" stop-opacity="0.1" />
              </linearGradient>
            </defs>
            <polyline class="heart-line" points="0,20 25,20 30,5 40,35 45,20 75,20 80,5 90,35 95,20 100,20" />
          </svg>
        </div>

        <h1 class="app-name">Mi<span>Heartbeat</span></h1>
<!--        <p class="tagline">BROADCASTING YOUR RHYTHM</p>-->

        <div class="progress-bar">
          <div class="progress-fill"></div>
        </div>
      </div>
      <div class="bottom-info">V1.0.1</div>
    </div>
  </div>
</template>

<script setup>
import { onMounted, ref } from 'vue';

const isFading = ref(false);

onMounted(() => {
  // 动画总时长 5s
  // 在 4.5 秒时开始淡出动画，持续 0.5 秒
  setTimeout(() => {
    isFading.value = true;
  }, 4500);

  // 在 5 秒时执行关闭或跳转逻辑
  setTimeout(() => {
    console.log("MiHeartbeat 初始化完成，准备进入主界面！");
  }, 5000);
});
</script>

<style scoped>
/* 全局复位，确保不出现滚动条 */
:global(body), :global(html) {
  margin: 0;
  padding: 0;
  overflow: hidden;
  background: transparent; /* 重要：让系统桌面透出来 */
}

/* 根容器：适配窗口，完全透明 */
.splash-screen-wrapper {
  width: 100vw;
  height: 100vh;
  display: flex;
  justify-content: center;
  align-items: center;
  background: transparent;
  user-select: none;
  /* 淡出动画 */
  transition: opacity 0.5s ease-out;
}

.fade-out {
  opacity: 0;
}

/* 模拟图片中的“窗口”效果 */
.splash-window {
  width: 90%; /* 响应式宽度，最大限制 */
  max-width: 480px;
  height: 70%; /* 响应式高度 */
  max-height: 300px;
  background: rgba(255, 255, 255, 0.15); /* 浅色半透明背景 */
  backdrop-filter: blur(20px); /* 核心毛玻璃效果 */
  -webkit-backdrop-filter: blur(20px); /* Webkit 兼容 */
  border-radius: 20px; /* 大圆角 */
  border: 1px solid rgba(255, 255, 255, 0.2); /* 轻微边框感 */
  box-shadow: 0 8px 32px 0 rgba(0, 0, 0, 0.1); /* 底部阴影，增加浮动感 */
  display: flex;
  flex-direction: column;
  justify-content: space-between; /* 内容上下均匀分布 */
  align-items: center;
  padding: 20px 30px; /* 内部填充 */
  color: #fff; /* 默认文字颜色 */
  font-family: 'Segoe UI', system-ui, sans-serif;
  position: relative;
  overflow: hidden; /* 防止内容溢出圆角 */
  -webkit-app-region: drag; /* 允许拖拽窗口 */
  animation: windowFadeIn 0.8s ease-out forwards; /* 窗口本身也有淡入效果 */
}

/* 顶部信息 */
.top-info {
  position: absolute;
  top: 15px;
  right: 20px;
  font-size: 0.7rem;
  letter-spacing: 1px;
  opacity: 0.6;
  text-transform: uppercase;
}

/* 核心内容区 */
.content-area {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  flex-grow: 1; /* 占据中心大部分空间 */
  width: 100%;
}

/* 心跳动画容器 */
.heart-pulse-container {
  width: 180px; /* 固定宽度，SVG会自适应 */
  height: 70px; /* 固定高度 */
  margin-bottom: 10px;
  animation: svgPulseScale 2.5s ease-in-out infinite alternate; /* 缩放呼吸感 */
}

.heart-line {
  fill: none;
  stroke: url(#heartGradient); /* 使用渐变填充 */
  stroke-width: 4; /* 线条粗细 */
  stroke-linecap: round;
  stroke-linejoin: round;
  stroke-dasharray: 250; /* 比线条实际长度略长 */
  stroke-dashoffset: 250;
  animation: drawHeart 2.5s cubic-bezier(0.4, 0, 0.2, 1) infinite; /* 心跳绘制动画 */
  filter: drop-shadow(0 0 8px rgba(255, 77, 77, 0.6)); /* 柔和的发光效果 */
}

/* 应用名称 */
.app-name {
  font-size: clamp(2rem, 6vw, 3rem); /* 响应式字体大小 */
  font-weight: 300;
  margin: 10px 0 5px 0;
  color: #1a1a1a; /* 深色文字，在毛玻璃上更清晰 */
  letter-spacing: -1px;
  line-height: 1.1;
  text-align: center;
  opacity: 0;
  animation: textSlideIn 1s ease-out 0.5s forwards; /* 文字淡入上滑 */
}

.app-name span {
  font-weight: 800;
  color: #ff4d4d; /* 红色高亮 */
}

/* 副标题 */
.tagline {
  font-size: clamp(0.6rem, 2vw, 0.8rem);
  letter-spacing: 3px;
  color: rgba(0, 0, 0, 0.6);
  margin-top: 5px;
  text-transform: uppercase;
  opacity: 0;
  animation: textFadeIn 1s ease-out 1s forwards; /* 文字淡入 */
}

/* 进度条 */
.progress-bar {
  position: relative;
  width: 80%; /* 响应式宽度 */
  max-width: 250px;
  height: 4px;
  background: rgba(0, 0, 0, 0.15); /* 轨道背景 */
  border-radius: 2px;
  overflow: hidden;
  margin-top: 30px;
}

.progress-fill {
  position: absolute;
  left: 0;
  top: 0;
  height: 100%;
  width: 0%;
  background: linear-gradient(90deg, #ff4d4d, #ff8c8c); /* 进度条渐变 */
  box-shadow: 0 0 8px rgba(255, 77, 77, 0.6); /* 进度条发光 */
  animation: fillProgress 5s cubic-bezier(0.1, 0, 0.3, 1) forwards; /* 填充动画 */
}

/* 底部版本信息 */
.bottom-info {
  font-size: 0.7rem;
  letter-spacing: 1px;
  color: rgba(0, 0, 0, 0.5);
  margin-top: 10px; /* 与进度条保持一定距离 */
}


/* 动画关键帧 */

@keyframes windowFadeIn {
  from { opacity: 0; transform: scale(0.95); }
  to { opacity: 1; transform: scale(1); }
}

@keyframes drawHeart {
  0% { stroke-dashoffset: 250; }
  50% { stroke-dashoffset: 0; }
  100% { stroke-dashoffset: -250; } /* 让线从左到右滑出 */
}

@keyframes svgPulseScale {
  0%, 100% { transform: scale(1); }
  50% { transform: scale(1.05); } /* 轻微放大缩小效果 */
}

@keyframes textSlideIn {
  from { opacity: 0; transform: translateY(10px); }
  to { opacity: 1; transform: translateY(0); }
}

@keyframes textFadeIn {
  from { opacity: 0; }
  to { opacity: 1; }
}

@keyframes fillProgress {
  0% { width: 0%; }
  100% { width: 100%; }
}
</style>