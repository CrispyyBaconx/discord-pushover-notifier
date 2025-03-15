import os
import logging
import asyncio
from typing import Optional, Dict, Any
import aiohttp

logger = logging.getLogger(__name__)

class PushoverClient:
    """Client for sending notifications to Pushover groups"""
    
    def __init__(self):
        """Initialize the Pushover client"""
        self.api_token = os.getenv("PUSHOVER_API_TOKEN")
        self.group_key = os.getenv("PUSHOVER_GROUP_KEY")
        self.api_url = "https://api.pushover.net/1/messages.json"
        self.enabled = self.api_token and self.group_key
        
        if not self.enabled:
            logger.warning("Pushover not configured. Set PUSHOVER_API_TOKEN and PUSHOVER_GROUP_KEY to enable.")
        else:
            logger.info("Pushover client initialized")
            
    async def push_to_group(self, content: Optional[str] = None, 
                           embed: Optional[Dict[str, Any]] = None,
                           priority: int = 0,
                           title: Optional[str] = None,
                           expire: int = 3600,
                           retry: int = 30) -> bool:
        """
        Send a notification to the configured Pushover group
        
        Args:
            content: Text content to send
            embed: Discord-style embed to convert to Pushover format
            priority: Message priority (-2 to 2, where 2 is emergency)
            title: Message title (optional)
            expire: Emergency notifications only - seconds to retry (max 10800)
            retry: Emergency notifications only - seconds between retries (min 30)
            
        Returns:
            bool: True if message was sent successfully, False otherwise
        """
        if not self.enabled:
            logger.debug("Pushover not configured. Skipping notification.")
            return False
            
        try:
            # Format message from content and/or embed
            message_text = self._format_message(content, embed)
            message_title = title or self._get_title_from_embed(embed)
            
            if not message_text:
                logger.warning("No content to send to Pushover")
                return False
                
            # Prepare data for API request
            data = {
                "token": self.api_token,
                "user": self.group_key,
                "message": message_text,
                "priority": priority
            }
            
            # Add title if provided
            if message_title:
                data["title"] = message_title
                
            # Add required parameters for emergency priority
            if priority == 2:
                data["expire"] = expire  # How long to retry (seconds)
                data["retry"] = retry    # How often to retry (seconds)
                logger.info(f"Sending emergency notification with expire={expire}s and retry={retry}s")
            
            # Send the notification
            async with aiohttp.ClientSession() as session:
                async with session.post(self.api_url, data=data) as response:
                    response_data = await response.json()
                    
                    if response.status == 200 and response_data.get("status") == 1:
                        logger.info(f"Pushover notification sent successfully")
                        return True
                    else:
                        logger.error(f"Failed to send Pushover notification: {response_data}")
                        return False
                        
        except Exception as e:
            logger.error(f"Error sending Pushover notification: {str(e)}")
            return False
    
    def _format_message(self, content: Optional[str], 
                        embed: Optional[Dict[str, Any]]) -> str:
        """
        Format Discord-style content and embeds for Pushover
        
        Args:
            content: Text content
            embed: Discord embed object
            
        Returns:
            str: Formatted message for Pushover
        """
        message_parts = []
        
        # Add content if provided
        if content:
            # Clean Discord-specific formatting
            clean_content = self._clean_discord_formatting(content)
            message_parts.append(clean_content)
        
        # Format embed for Pushover if provided
        if embed:
            # Add description
            if "description" in embed:
                # Clean Discord formatting from description
                clean_description = self._clean_discord_formatting(embed['description'])
                message_parts.append(clean_description)
            
            # Add fields if present
            if "fields" in embed and isinstance(embed["fields"], list):
                for field in embed["fields"]:
                    if "name" in field and "value" in field:
                        clean_name = self._clean_discord_formatting(field['name'])
                        clean_value = self._clean_discord_formatting(field['value'])
                        message_parts.append(f"{clean_name}: {clean_value}")
            
            # Add footer if present
            if "footer" in embed and isinstance(embed["footer"], dict) and "text" in embed["footer"]:
                footer_text = self._clean_discord_formatting(embed["footer"]["text"])
                message_parts.append(f"{footer_text}")
        
        # Join all parts with double newlines
        formatted_message = "\n\n".join(message_parts)
        
        # Truncate if needed (Pushover has a 1024 character limit)
        if len(formatted_message) > 1024:
            formatted_message = formatted_message[:1021] + "..."
            logger.warning("Message truncated to 1024 characters for Pushover")
            
        return formatted_message
        
    def _clean_discord_formatting(self, text: str) -> str:
        """
        Clean Discord-specific formatting from text
        
        Args:
            text: The text to clean
            
        Returns:
            str: Cleaned text safe for Pushover
        """
        import re
        
        if text is None:
            return ""
            
        # Replace Discord role mentions <@&ROLE_ID> with "Traders"
        text = re.sub(r'<@&\d+>', 'Traders', text)
        
        # Replace Discord user mentions <@USER_ID> with "User"
        text = re.sub(r'<@\d+>', 'User', text)
        
        # Replace Discord channel mentions <#CHANNEL_ID> with "channel"
        text = re.sub(r'<#\d+>', 'channel', text)
        
        # Replace Discord emojis <:emoji:ID> with their name
        text = re.sub(r'<:[a-zA-Z0-9_]+:\d+>', lambda m: m.group(0).split(':')[1], text)
        
        return text 
    
    def _get_title_from_embed(self, embed: Optional[Dict[str, Any]]) -> Optional[str]:
        """Extract title from an embed if available"""
        if embed and "title" in embed:
            return embed["title"]
        return None 