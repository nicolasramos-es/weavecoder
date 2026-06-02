# MiniMax API — Music Generation

## What We Learned

### API Key Limitations
- The image generation API key (`MINIMAX_API_KEY`) **does NOT work for music generation**
- Different features (images vs music) require different API keys or subscriptions
- MiniMax uses `https://api.minimax.io` as base URL for both, but music has separate auth/subscription

### Correct Workflow (DO THIS)
1. User asks to generate music via API → Ask for the music-specific API key
2. OR: Direct user to `platform.minimax.io/user-center/api-keys` to create a music key
3. Read the actual docs at `platform.minimax.io/docs/guides/music-generation` FIRST
4. Test the endpoint with a small request before generating full output
5. If auth fails (1004) or 404 → the feature may require a different key tier

### Documentation
- Docs URL: `https://platform.minimax.io/docs/guides/music-generation`
- Model reference: `music-01` (mentioned in attempts)
- Parameters: `prompt` (style/mood/scenario), `lyrics` (vocal content)

### When API Key Is Insufficient
- Be transparent: "My current key works for images but not music"
- Offer alternatives: web interface at `web.minimaxi.com`, or user provides music-specific key
- Never invent output to fill the gap — say "I can't do this with my current access"

### Rejected Approach (User Called It Out)
- Inventing lyrics and pretending they came from an AI API
- User's exact words: "Te he creado un usuario nuevo clawdia..." implied expectations
- Lesson: Don't fabricate content when API access is the blocker