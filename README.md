# Scallion Vocab

目標是打造 Anki 相容的 Material Design 3 風格安卓背單字 APP。

## TODO

- [ ] Anki 牌組同步
- [ ] 文字轉語音 (TTS)
- [ ] 演算法 (FSRS)
- [ ] 自訂牌組分類與標籤
- [ ] 考試模式選擇（選擇題 / 拼寫 / 聽寫）
- [ ] 更多設定選項
- [ ] 進度統計圖表
- [ ] Windows/Linux 桌面端

## Build

需要 Podman/Docker (可在 scripts/env.txt 設定)
容器將自動配置好編譯環境

```bash
bash scripts/rebuild-container.sh   # 建立 build container
bash scripts/build-android.sh       # 編譯 APK（輸出在 build/scallion-vocab.apk）
```


## License

[AGPL v3](https://raw.githubusercontent.com/HongyiHank/scallion-vocab/refs/heads/main/LICENSE)
