#!/bin/bash
set -euxo pipefail

# 本地测试 Gitee API 调用格式
# 用法: GITEE_TOKEN=xxx GITEE_REPO=anxi-yizhi/dev-claw ./test-gitee-api.sh

if [ -z "${GITEE_TOKEN:-}" ] || [ -z "${GITEE_REPO:-}" ]; then
  echo "❌ 请设置环境变量:"
  echo "   export GITEE_TOKEN=your_token"
  echo "   export GITEE_REPO=anxi-yizhi/dev-claw"
  exit 1
fi

TAG="v0.1.4"
OWNER="${GITEE_REPO%%/*}"
REPO="${GITEE_REPO#*/}"

echo "🔧 测试配置:"
echo "   Owner: ${OWNER}"
echo "   Repo: ${REPO}"
echo "   Tag: ${TAG}"
echo ""

# 测试 1: 检查仓库是否存在
echo "📋 测试 1: 检查仓库访问权限..."
HTTP_CODE=$(curl -s -w "\n%{http_code}" -o /tmp/gitee_repo.json \
  "https://gitee.com/api/v5/repos/${GITEE_REPO}?access_token=${GITEE_TOKEN}")

echo "HTTP status: ${HTTP_CODE}"
if [ "${HTTP_CODE}" -eq 200 ]; then
  echo "✅ 仓库访问成功"
  REPO_NAME=$(python3 -c "import json; print(json.load(open('/tmp/gitee_repo.json'))['name'])")
  echo "   仓库名: ${REPO_NAME}"
else
  echo "❌ 仓库访问失败"
  cat /tmp/gitee_repo.json
  exit 1
fi

echo ""

# 测试 2: 检查 tag 是否存在
echo "📋 测试 2: 检查 tag ${TAG} 是否存在..."
HTTP_CODE=$(curl -s -w "\n%{http_code}" -o /tmp/gitee_tag.json \
  "https://gitee.com/api/v5/repos/${GITEE_REPO}/tags/${TAG}?access_token=${GITEE_TOKEN}")

echo "HTTP status: ${HTTP_CODE}"
if [ "${HTTP_CODE}" -eq 200 ]; then
  echo "✅ Tag ${TAG} 存在"
else
  echo "⚠️  Tag ${TAG} 不存在，需要先创建"
  echo "   响应:"
  cat /tmp/gitee_tag.json
fi

echo ""

# 测试 3: 尝试创建 release (dry-run 检查)
echo "📋 测试 3: 测试创建 release 的请求格式..."
echo "   请求 URL: https://gitee.com/api/v5/repos/${GITEE_REPO}/releases?access_token=***"
echo "   请求体:"
cat <<EOF
{
  "tag_name": "${TAG}",
  "name": "DevClaw ${TAG}",
  "body": "Release ${TAG}",
  "prerelease": false
}
EOF

echo ""
echo "🔍 实际发送请求..."

HTTP_CODE=$(curl -s -w "\n%{http_code}" -o /tmp/gitee_release.json \
  -X POST "https://gitee.com/api/v5/repos/${GITEE_REPO}/releases?access_token=${GITEE_TOKEN}" \
  -H "Content-Type: application/json" \
  -d "{
    \"tag_name\": \"${TAG}\",
    \"name\": \"DevClaw ${TAG}\",
    \"body\": \"Release ${TAG}\",
    \"prerelease\": false
  }")

echo "HTTP status: ${HTTP_CODE}"
echo "响应:"
cat /tmp/gitee_release.json

if [ "${HTTP_CODE}" -ge 200 ] && [ "${HTTP_CODE}" -lt 300 ]; then
  echo ""
  echo "✅ Release 创建成功!"
  RELEASE_ID=$(python3 -c "import json; print(json.load(open('/tmp/gitee_release.json'))['id'])")
  echo "   Release ID: ${RELEASE_ID}"

  # 测试 4: 上传附件
  echo ""
  echo "📋 测试 4: 测试上传附件..."
  echo "   创建测试文件..."
  echo "test content" > /tmp/test-upload.txt

  HTTP_CODE=$(curl -s -w "\n%{http_code}" -o /tmp/gitee_upload.json \
    -X POST "https://gitee.com/api/v5/repos/${GITEE_REPO}/releases/${RELEASE_ID}/attach_files?access_token=${GITEE_TOKEN}" \
    -H "Content-Type: multipart/form-data" \
    -F "file=@/tmp/test-upload.txt")

  echo "HTTP status: ${HTTP_CODE}"
  echo "响应:"
  cat /tmp/gitee_upload.json

  if [ "${HTTP_CODE}" -ge 200 ] && [ "${HTTP_CODE}" -lt 300 ]; then
    echo ""
    echo "✅ 附件上传成功!"
    DOWNLOAD_URL=$(python3 -c "import json; print(json.load(open('/tmp/gitee_upload.json')).get('browser_download_url','N/A'))")
    echo "   下载链接: ${DOWNLOAD_URL}"
  else
    echo "❌ 附件上传失败"
  fi
else
  echo ""
  echo "❌ Release 创建失败"
  echo "   常见原因:"
  echo "   - Tag 不存在 (需要先推送 tag)"
  echo "   - Release 已存在"
  echo "   - Token 权限不足"
fi

echo ""
echo "🧹 清理临时文件..."
rm -f /tmp/gitee_*.json /tmp/test-upload.txt

echo ""
echo "✨ 测试完成!"
