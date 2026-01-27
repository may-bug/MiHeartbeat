import { platform } from '@tauri-apps/plugin-os';
import OTHERICON from '../assets/images/other.png';

const extractBrandAndModel = (deviceName) => {
    if (!deviceName) return null;
    const nameLower = deviceName.toLowerCase();
    const brands = ['xiaomi', 'apple', 'samsung', 'garmin', 'fitbit', 'huawei', 'oppo', 'vivo', 'realme'];
    let matchedBrand = null;
    for (const brand of brands) {
        if (nameLower.includes(brand)) {
            matchedBrand = brand;
            break;
        }
    }

    if (!matchedBrand) return null;
    const brandIndex = nameLower.indexOf(matchedBrand);
    const afterBrand = deviceName.substring(brandIndex + matchedBrand.length).trim();
    const modelMatch = afterBrand.match(/(\d+[\s\w]*)/);
    const model = modelMatch ? modelMatch[0].trim() : afterBrand;

    return {
        brand: matchedBrand,
        model: model,
        fullModel: model ? `${matchedBrand} ${model}` : matchedBrand
    };
};

const matchDeviceImage = async (deviceName) => {
    if (!deviceName) return null;

    try {
        const imageModules = import.meta.glob('../assets/images/*.png', { eager: true });

        if (Object.keys(imageModules).length === 0) {
            return null;
        }

        const deviceInfo = extractBrandAndModel(deviceName);
        if (!deviceInfo) return null;

        const nameLower = deviceName.toLowerCase();
        const brandLower = deviceInfo.brand.toLowerCase();
        const modelLower = deviceInfo.model.toLowerCase();
        for (const filePath of Object.keys(imageModules)) {
            const fileName = filePath.split('/').pop().replace('.png', '').toLowerCase();
            if (fileName === nameLower ||
                fileName === deviceInfo.fullModel.toLowerCase() ||
                (fileName.includes(brandLower) && fileName.includes(modelLower.split(' ')[0]))) {
                return imageModules[filePath].default;
            }
        }
        for (const filePath of Object.keys(imageModules)) {
            const fileName = filePath.split('/').pop().replace('.png', '').toLowerCase();
            const firstModelPart = modelLower.split(/[\s+]/)[0];

            if (fileName.includes(brandLower) && fileName.includes(firstModelPart)) {
                return imageModules[filePath].default;
            }
        }

        return null;
    } catch (error) {
        console.error("Error matching device image:", error);
        return null;
    }
};

const getDeviceIcon = async (deviceName) => {
    if (!deviceName) return OTHERICON;

    try {
        const imagePath = await matchDeviceImage(deviceName);
        return imagePath || OTHERICON;
    } catch (error) {
        console.error("Error getting device icon:", error);
        return OTHERICON;
    }
};

// 判断是否为图片路径
const isImagePath = (icon) => {
    return typeof icon === 'string' && icon.includes('.png');
};

// 获取平台图标
const getPlatformIcon = async (platformType) => {
    if (!platformType) return '';

    try {
        const iconPath = `../assets/icons/${platformType}.svg`;
        const imageModules = await import.meta.glob('../assets/icons/*.svg', { eager: true });
        return await imageModules[iconPath]?.default || '';
    } catch (error) {
        console.error("Error getting platform icon:", error);
        return '';
    }
};

export { getDeviceIcon, isImagePath, getPlatformIcon };