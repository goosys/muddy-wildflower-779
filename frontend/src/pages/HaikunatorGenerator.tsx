'use client'

import { useState } from 'react'

export const HaikunatorGenerator = () => {
  const [generatedString, setGeneratedString] = useState<string>('')

  const generateString = async () => {
    try {
      const response = await fetch('/api/gen')
      if (response.ok) {
        const data: { name: string } = await response.json()
        setGeneratedString(data.name)
      } else {
        throw new Error('Failed to generate string')
      }
    } catch (error) {
      console.error(error)
    }
  }

  return (
    <div className="min-h-screen flex flex-col items-center justify-center bg-gray-100 p-4">
      <header className="mt-auto mb-4 text-right text-gray-500 fixed top-2 right-4">
        <p><a href="https://github.com/goosys/muddy-wildflower-779">Contact</a></p>
      </header>

      <main className="w-full max-w-lg bg-white rounded-lg shadow-md p-8 flex flex-col items-center space-y-6">
        <h1 className="text-2xl font-bold text-gray-800 text-center">Haikunator Generator</h1>
        <p className="text-gray-600 text-center mt-2">Heroku-like memorable random string</p>

        <div className="w-full bg-gray-100 p-2 sm:p-4 rounded-md flex items-center">
            <div className="text-center flex-grow pl-4">
            {generatedString ? (
              <p className="text-xl sm:text-2xl font-bold text-center break-all">{generatedString}</p>
            ) : (
              <p className="text-gray-400 text-xl sm:text-2xl font-bold text-center break-all">muddy-wildflower-779</p>
            )}
            </div>
          <button
            onClick={(e) => {
              navigator.clipboard.writeText(generatedString)
            }}
            className={`text-gray-600 hover:text-gray-800 focus:outline-none ${generatedString ? 'visible' : 'invisible'}`}
          >
            <svg xmlns="http://www.w3.org/2000/svg" width="32" height="32" viewBox="0 0 24 24"><path fill="currentColor" d="M16 1H4c-1.1 0-2 .9-2 2v14h2V3h12zm3 4H8c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h11c1.1 0 2-.9 2-2V7c0-1.1-.9-2-2-2m0 16H8V7h11z"/></svg>
          </button>
        </div>

        <button
          onClick={generateString}
          className="px-6 py-3 bg-blue-600 text-white rounded-md hover:bg-blue-700 transition-colors duration-300 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-opacity-50"
        >
          Generate
        </button>
      </main>

      <footer className="mt-auto mb-4 text-center text-gray-500 fixed bottom-0 left-0 right-0">
        <p><a href="https://github.com/nishanths/rust-haikunator">Powered by Haikunator</a></p>
      </footer>
    </div>
  )
}
