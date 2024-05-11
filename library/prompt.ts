export function addInstruct(prompt: string): string {
    return prompt + ' Furthermore, could you make sure that this is actually done in JavaScript instead, with a simple test in the code itself using console.log statments?';
    //return prompt + ' Furthermore, could you make sure that this is actually done in JavaScript instead? And could you make sure that, in your response, you give ONLY code (no text or explanation, except in CODE comments), with a simple test in the code itself using console.log statments?';
}

export async function prompt(prompt: string, apiKey: string): Promise<string> {
    const requestData = {
        model: 'gpt-3.5-turbo',
        messages: [{ role: 'user', content: prompt}],
        temperature: 0.7,
    };

    let retries = 3; // Retry up to 3 times
    while (retries > 0) {
        try {
            const response = await fetch('https://api.openai.com/v1/chat/completions', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                    Authorization: `Bearer ${apiKey}`,
                },
                body: JSON.stringify(requestData),
                });
            
            let code = '';
            if (response.ok) {
                const responseData = await response.json();
                code = responseData.choices[0].message.content;
            } else {
                console.error('Failed to fetch data:', response.status, response.statusText);
                code = "Failed to fetch data: " + response.status + " " + response.statusText;
            }
            return code;
        } catch (error) {
            console.error('Error fetching data:', error);
            retries--;
        }
    }
    throw new Error('Failed to fetch data after multiple retries');
}
