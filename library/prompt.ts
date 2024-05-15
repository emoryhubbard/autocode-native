
export async function prompt(prompt: string, apiKey: string): Promise<string> {
    let retries = 3; // Retry up to 3 times
    while (retries > 0) {
        try {
            const MODEL = {} as any;
            /*const response = await fetch('https://api.openai.com/v1/chat/completions', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                    Authorization: `Bearer ${apiKey}`,
                },
                body: JSON.stringify({
                    model: 'gpt-3.5-turbo',
                    messages: [{ role: 'user', content: prompt}],
                    temperature: 0.7,
                }),
                });*/
            
            let code = '';
            if (MODEL.ok) {
                const responseData = await MODEL.json();
                code = responseData.choices[0].message.content;
            } else {
                console.error('Failed to fetch data:', MODEL.status, MODEL.statusText);
                code = "Failed to fetch data: " + MODEL.status + " " + MODEL.statusText;
            }
            return code;
        } catch (error) {
            console.error('Error fetching data:', error);
            retries--;
        }
    }
    throw new Error('Failed to fetch data after multiple retries');
}
