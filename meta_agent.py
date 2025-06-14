#!/usr/bin/env python3
import os
import sys
import json
import argparse
from dotenv import load_dotenv
import google.genai as genai
from google.genai import types
from typing import Dict, Any, Optional
import subprocess
from datetime import datetime
import asyncio

class MetaAgent:
    def __init__(self):
        self.client = genai.Client("AIzaSyAd4XP6jBlhoOVCrwyuS-10eRW_1-10ess")
        self.model = "gemini-2.5-flash-preview-05-20"
        self.cursor_position = 0
        self.conversation_history = []

    def get_full_context(self, cursor_position: Optional[int] = None) -> str:
        """Get full context including git history, test results, and cursor position"""
        context_parts = []
        
        # Get git context
        try:
            git_result = subprocess.run(["git2gpt", "."], capture_output=True, text=True, check=True)
            context_parts.append("Git Context:")
            context_parts.append(git_result.stdout)
        except Exception as e:
            print(f"Error getting git context: {e}")
        
        # Get test results
        try:
            test_result = subprocess.run(
                ["cargo", "test", "--no-fail-fast", "--", "--nocapture"],
                capture_output=True,
                text=True,
                check=False
            )
            context_parts.append("\nTest Results:")
            context_parts.append(test_result.stdout)
            if test_result.stderr:
                context_parts.append("\nTest Errors:")
                context_parts.append(test_result.stderr)
        except Exception as e:
            print(f"Error getting test results: {e}")

        # Add cursor position if provided
        if cursor_position is not None:
            context_parts.append(f"\nCursor Position: {cursor_position}")
        
        return "\n".join(context_parts)

    async def make_api_request(self, prompt: str) -> Dict[str, Any]:
        """Make request to Gemini API"""
        print("Making API request...")
        contents = [
            types.Content(
                role="user",
                parts=[types.Part(text=prompt)],
            ),
        ]
        config = types.GenerateContentConfig(
            response_mime_type="application/json",
            temperature=0.2,
            top_p=0.8,
            top_k=40,
            max_output_tokens=65536,
        )
        try:
            response_text = ""
            for chunk in self.client.models.generate_content_stream(
                model=self.model,
                contents=contents,
                config=config,
            ):
                response_text += chunk.text
                print(chunk.text, end="", flush=True)
            print()
            
            return json.loads(response_text)
        except Exception as e:
            print(f"Error making API request: {e}", file=sys.stderr)
            return {"error": str(e)}

    async def run(self, task_description: str, cursor_position: Optional[int] = None) -> None:
        """Run the meta agent with the given task"""
        print("Starting meta agent run...")
        
        # Update cursor position
        if cursor_position is not None:
            self.cursor_position = cursor_position
        
        # Add to conversation history
        self.conversation_history.append({
            "role": "user",
            "content": task_description,
            "cursor_position": self.cursor_position,
            "timestamp": datetime.now().isoformat()
        })
        
        # Get all context
        context = self.get_full_context(self.cursor_position)
        
        # Create analysis prompt with full context
        prompt = f"""You are a META agent for Cursor IDE, specialized in Rust development.
Your task is to analyze the following request and provide insights.

Task Description: {task_description}

Full Context:
{context}

Conversation History:
{json.dumps(self.conversation_history, indent=2)}

Please provide your analysis and suggestions in JSON format:
{{
    "analysis": {{
        "issues": ["list of issues found"],
        "suggestions": ["list of suggestions"],
        "next_steps": ["recommended next steps"]
    }},
    "cursor_action": {{
        "type": "move|select|edit|none",
        "position": number,
        "selection": {{
            "start": number,
            "end": number
        }},
        "edit": {{
            "type": "insert|delete|replace",
            "content": "content to insert/replace"
        }}
    }},
    "confidence": 0.0-1.0,
    "requires_human_input": true|false
}}"""

        # Get analysis from LLM
        response = await self.make_api_request(prompt)
        
        # Add response to conversation history
        self.conversation_history.append({
            "role": "assistant",
            "content": response,
            "timestamp": datetime.now().isoformat()
        })
        
        print("\nAnalysis complete!")
        
        # If cursor action is provided, update cursor position
        if "cursor_action" in response and response["cursor_action"]["type"] != "none":
            self.cursor_position = response["cursor_action"].get("position", self.cursor_position)
            print(f"\nCursor moved to position: {self.cursor_position}")

async def main():
    parser = argparse.ArgumentParser(
        description='Meta Agent for code analysis',
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  python3 meta_agent.py "analyze this code"
  python3 meta_agent.py "fix this function" --cursor 123
  python3 meta_agent.py "please analyze the problems and give fixes" --cursor 0

Note: Always wrap your task description in quotes if it contains spaces!
"""
    )
    parser.add_argument('task', help='Task description for the meta agent (wrap in quotes if it contains spaces)')
    parser.add_argument('--cursor', type=int, help='Current cursor position', default=None)
    args = parser.parse_args()

    agent = MetaAgent()
    await agent.run(args.task, args.cursor)

if __name__ == "__main__":
    try:
        asyncio.run(main())
    except KeyboardInterrupt:
        print("\nOperation cancelled by user")
    except Exception as e:
        print(f"\nError: {str(e)}")
        print("\nUsage: python3 meta_agent.py \"your task description\" [--cursor POSITION]")
        print("Note: Always wrap your task description in quotes if it contains spaces!")
        sys.exit(1) 