'use client'

import { useState } from 'react'

export const HaikunatorGenerator = () => {
  const [generatedString, setGeneratedString] = useState<string>('')

  const generateString = () => {
    const randomString = Math.random().toString(36).substring(7)
    setGeneratedString(randomString)
  }

  return (
    <div className="min-h-screen flex items-center justify-center bg-gray-100 p-4">
      <div className="w-full max-w-md bg-white rounded-lg shadow-md p-8 flex flex-col items-center space-y-6">
        <h1 className="text-2xl font-bold text-gray-800 text-center">文字列ジェネレーター</h1>
        <div className="w-full bg-gray-100 p-4 rounded-md">
          <p className="text-center break-all">{generatedString}</p>
        </div>
        <button
          onClick={generateString}
          className="px-6 py-3 bg-blue-600 text-white rounded-md hover:bg-blue-700 transition-colors duration-300 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-opacity-50"
        >
          文字列を生成
        </button>
      </div>
    </div>
  )
}
